extern crate lazy_static;

pub mod application;
pub mod domain;
pub mod infrastructure;
use crate::application::cli::{Cli, Commands};
use crate::domain::digesters::Sha256Digest;
use crate::domain::services::media::DefaultMediaService;
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = Cli::parse();

    if !std::env::vars().any(|(k, _)| k == "RUST_LOG") {
        std::env::set_var("RUST_LOG", "hyper=warn,debug");
    }

    env_logger::init();
    let result = match &args.command {
        Commands::Analyze {
            folder,
            verbose,
            delete,
            media_type,
        } => analyze_media_folder(folder, verbose, delete, media_type),
        Commands::Info { folder, verbose } => get_info(folder, verbose),
    };

    println!("{:?}", result);
    result
}

fn analyze_media_folder(
    folder: &str,
    verbose: &bool,
    _delete: &bool,
    media_type: &str,
) -> Result<()> {
    let digester = Sha256Digest {};
    //let memory_store = MemoryStore::<String, LinkedList<FileMetadata>>::new();
    let mut media_service = DefaultMediaService::new(folder, digester, verbose.to_owned());
    media_service.analyze_folder(media_type)
}

fn get_info(folder: &str, verbose: &bool) -> Result<()> {
    let digester = Sha256Digest {};
    //let memory_store = MemoryStore::<String, LinkedList<FileMetadata>>::new();

    let media_service = DefaultMediaService::new(folder, digester, verbose.to_owned());
    let info = media_service.get_info()?;

    println!("Images found:  {}", info.images.count);
    println!("Images Size: {} MB", info.images.size / 1024 / 1024);
    println!("Videos found:  {}", info.videos.count);
    println!("Videos Size: {} MB", info.videos.size / 1024 / 1024);
    println!("Elapsed Time: {} ms", info.elapsed_time);

    Ok(())
}
