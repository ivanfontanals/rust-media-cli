use crate::domain::model::DigestDto;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[async_trait(?Send)]
pub trait MediaService: Send {
    fn analyze_folder(&self, recursive: bool) -> Result<()>;
}

#[async_trait(?Send)]
pub trait Digester: Send {
    fn digest<P: AsRef<Path>>(&self, path: P) -> Result<DigestDto>;
}

#[async_trait(?Send)]
pub trait ImageHashing: Send {
    fn digest<P: AsRef<Path>>(&self, path: P) -> Result<Vec<DigestDto>>;
    fn distance(&self, hash1: &str, hash2: &str) -> Result<u32>;
}
