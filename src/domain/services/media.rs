use crate::domain::digesters::phash::PHashImageHashing;
use crate::domain::model::{AnalysisResponseDto, FileMetadata, FileType, MediaInfoDto, DigestDto};
use crate::domain::ports::incoming::{Digester, ImageHashing};
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::{debug, info};
use regex::Regex;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::fs::metadata;
use std::rc::Rc;
use std::time::Instant;
use walkdir::{DirEntry, WalkDir};

static IMAGES_REGEX_PATTERN: &str = r"\.(gif|jpe?g|bmp|png)$";

lazy_static! {
    static ref IMAGES_REGEX: Regex = Regex::new(IMAGES_REGEX_PATTERN).unwrap();
}

pub struct DefaultMediaService<D> {
    digester: D,
    path: String,
    verbose: bool,
    duplications_map: HashMap::<String, bool>,
    store_hash_map: HashMap<String, LinkedList<Rc<FileMetadata>>>,
    directory_map: HashMap<String, LinkedList<Rc<FileMetadata>>>,
}



impl<D> DefaultMediaService<D>
where
    D: Digester + Send + Sync + 'static,
{
    pub fn new(folder: &str, digester: D, verbose: bool) -> Self {
        DefaultMediaService {
            digester,
            path: folder.to_string(),
            verbose,
            duplications_map: HashMap::new(),
            store_hash_map: HashMap::new(),
            directory_map:  HashMap::new(),
        }
    }

    fn is_matching_pattern(entry: &DirEntry, re: Regex) -> bool {
        entry
            .metadata()
            .map(|metadata| {
                entry
                    .file_name()
                    .to_str()
                    .map(|s| metadata.is_dir() || re.is_match(s))
            })
            .map(|pattern_match| pattern_match.unwrap())
            .unwrap_or(false)
    }

    fn get_file_metadata(&self, entry: &DirEntry) -> Result<FileMetadata> {
        let metadata = entry.metadata()?;
        let file_path = entry.path().display().to_string();

        let file_type = if IMAGES_REGEX.is_match(&file_path) {
            FileType::IMAGE
        } else {
            FileType::OTHER
        };

        let (directory, file_name) = match file_path.rfind('/') {
            Some(last_slash_pos) => {
                let filename = file_path[(last_slash_pos + 1)..].to_string();
                let path = file_path[..(last_slash_pos + 1)].to_string();
                (Some(path), filename)
            }
            None => (None, file_path),
        };

        Ok(FileMetadata {
            path: entry.path().display().to_string(),
            folder: directory,
            name: file_name,
            is_dir: metadata.is_dir(),
            size: metadata.len() as u64,
            file_type,
            phash: None,
        })
    }

    pub fn get_info(&self) -> Result<AnalysisResponseDto> {
        let now = Instant::now();
        let metadata = metadata(&self.path)?;

        let images_suffix = Regex::new(r"\.(gif|jpe?g|bmp)$")?;
        let video_suffix = Regex::new(r"\.(mpg|avi|mp4)$")?;

        if !metadata.is_dir() {
            return Err(anyhow!("The path {} is not a directory", &self.path));
        }

        Ok(AnalysisResponseDto {
            images: self.get_folder_info(&images_suffix)?,
            videos: self.get_folder_info(&video_suffix)?,
            elapsed_time: now.elapsed().as_millis(),
        })
    }

    fn get_folder_info(&self, regex: &Regex) -> Result<MediaInfoDto> {
        let mut count = 0;
        let mut total_size: u64 = 0;

        for entry in WalkDir::new(&self.path)
            .into_iter()
            .filter_entry(|e| Self::is_matching_pattern(e, regex.clone()))
            .filter_map(|e| e.ok())
        {
            if let Ok(metadata) = self.get_file_metadata(&entry) {
                if !metadata.is_dir {
                    if self.verbose {
                        debug!(target:"media","Checking path:  {}", entry.path().display());
                    }
                    count += 1;
                    total_size += metadata.size;
                };
            }
        }
        Ok(MediaInfoDto {
            size: total_size,
            count,
        })
    }

    fn create_perceptual_hash(&self, metadata: &FileMetadata) -> Option<Vec<DigestDto>> {
        if metadata.file_type == FileType::IMAGE {
            if self.verbose {
                debug!("Creating perceptual hash for: {}", metadata.path);
            }
            let phash_image = PHashImageHashing {};
            phash_image.digest(metadata.path.as_str()).ok()
        } else {
            None
        }
    }

    fn update_directory_index(
        &mut self,
        directory: &str,
        metadata: Rc<FileMetadata>,
    ) {
        match self.directory_map.get_mut(directory) {
            Some(list) => {
                list.push_back(metadata);
            }
            None => {
                let mut list = LinkedList::new();
                list.push_back(metadata);
                self.directory_map.insert(directory.to_string(), list);
            }
        };
    }

    pub fn analyze_folder(&mut self, media_type: &str) -> Result<()> {
        let now = Instant::now();

       

       let metadata = metadata(&self.path)
            .map_err(|x| anyhow!("Invalid path {}: {}", &self.path, x.to_string()))?;

        if !metadata.is_dir() {
            return Err(anyhow!("The path {} is not a directory", &self.path));
        }

        let suffix_regexp = Regex::new(match media_type {
            "all" => r"\.(mpg|avi|mp4|gif|jpe?g|bmp|png)$",
            "images" => IMAGES_REGEX_PATTERN,
            "video" => r"\.(mpg|avi|mp4)$",
            _ => {
                return Err(anyhow!(
                    "Invalid media type. Supported media types are all, images and video"
                ))
            }
        })?;

        let mut count = 0;
        let mut duplications_counter = 0;
        let mut total_size: u64 = 0;

        for entry in WalkDir::new(self.path.clone())
            .into_iter()
            .filter_entry(|e| Self::is_matching_pattern(e, suffix_regexp.clone()))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if let Ok(metadata) = self.get_file_metadata(&entry) {
                if !&metadata.is_dir {
                    let directory = &metadata.folder.clone().unwrap_or_else(|| "/".to_string());
                 
                    let ref_to_metadata = Rc::new(FileMetadata{
                        phash: self.create_perceptual_hash(&metadata),
                        ..metadata
                    });
                    
                    if let Ok(digest) = self.digester.digest(path) {
                        let key = digest.value.clone();
                        //Update hash index
                        match self.store_hash_map.get_mut(&key) {
                            Some(duplicates) => {
                                duplicates.push_back(ref_to_metadata.clone());
                                self.duplications_map.insert(key, true);
                                duplications_counter += 1;
                            }
                            None => {
                                let mut list = LinkedList::<Rc<FileMetadata>>::new();
                                list.push_back(ref_to_metadata.clone());
                                self.store_hash_map.insert(key, list);
                            }
                        };

                        //Update directory index
                        self.update_directory_index(
                            directory,
                            ref_to_metadata.clone(),
                        );

                        if self.verbose {
                            debug!("path: {}, {:?}", entry.path().display(), digest.value);
                        }
                        count += 1;
                        total_size += metadata.size;

                        if count % 100 == 0 {
                            info!(target:"digester","Number of files processed:  {} in {} ms", count, now.elapsed().as_millis() );
                        }
                    };
                }
            }
        }
        let phash_image = PHashImageHashing {};
        for (key, value) in self.directory_map.iter() {
            println!("Directory: {}", key);
            for item1 in value {
                println!("    {}, phash: {:?}", &item1.name, item1.phash);
                
                for item2 in value {
                    if item1.name!=item2.name {
                        if let (Some(vector1),Some(vector2)) = (&item1.phash, &item2.phash)  {
                            for (pos, first) in vector1.iter().enumerate() {
                                let second = &vector2[pos];
                                let distance= phash_image.distance(&first.value, &second.value).ok();
                                println!("Distance between  {} and {} is: {:?} [{}]", &item1.name, &item2.name, distance, first.algorithm);
                            }
                        };
/*
                        if let (Some(hash1), Some(hash2)) = (&item1.phash, &item2.phash) {
                            let distance= phash_image.distance(&hash1.value, &hash2.value).ok();
                            println!("Distance beweened  {} and {} is: {:?}", &item1.name, &item2.name, distance);
                        }*/
                    }
                }
            }
        }

        for (key, _value) in self.duplications_map.iter() {
            println!("Duplicated file found with hash {}", key);
            if let Some(list) = self.store_hash_map.get(key) {
                for item in list {
                    println!("    {}", item.path);
                }
            };
        }

        info!(target:"digester","Number of files processed:  {} in {} ms", count, now.elapsed().as_millis() );
        info!(target:"digester","Number of duplicated files found:  {}", duplications_counter );
        info!(target:"digester","Total Size: {} MB", total_size/1024/1024);

        info!(target:"digester","Rc<FileMetadata>> Size: {} ", std::mem::size_of::<Rc<FileMetadata>>());
        info!(target:"digester","FileMetadata Size: {} ", std::mem::size_of::<FileMetadata>());
        info!(target:"digester","String Size: {} ", std::mem::size_of::<String>());
        let mut total_memory_usage =
            count * (std::mem::size_of::<Rc<FileMetadata>>() + std::mem::size_of::<FileMetadata>());
        total_memory_usage += std::mem::size_of::<String>() * count; // Hash size
        total_memory_usage += std::mem::size_of::<String>() * duplications_counter; // Duplicated Hash size

        info!(target:"digester","Aproximate Size: {} ", total_memory_usage);
        Ok(())
    }
}


