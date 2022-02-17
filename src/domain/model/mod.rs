use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct FileMetadata {
    pub path: String,
    pub folder: Option<String>,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub file_type: FileType,
    pub phash: Option<Vec<DigestDto>>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub enum FileType {
    VIDEO,
    IMAGE,
    OTHER,
}



pub struct MediaInfoDto {
    pub count: u64,
    pub size: u64,
}

pub struct AnalysisResponseDto {
    pub images: MediaInfoDto,
    pub videos: MediaInfoDto,
    pub elapsed_time: u128,
}

#[derive(Default, Clone, Debug, Serialize)]
pub struct DigestDto {
    pub algorithm: String,
    pub value: String,
    pub elapsed_time: u128,
}
