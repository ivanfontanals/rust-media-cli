use crate::domain::model::DigestDto;
use crate::domain::ports::incoming::ImageHashing;
use anyhow::{anyhow, Result};
use img_hash::{image, HashAlg, HasherConfig, ImageHash};
use image::DynamicImage;
use std::path::Path;
use std::time::Instant;

pub struct PHashImageHashing {}


impl PHashImageHashing {

    fn create_hash(&self, image: &DynamicImage, alg: HashAlg ) -> DigestDto {
        let now = Instant::now();
        let hasher = HasherConfig::new()
            .hash_alg(alg)
            .to_hasher();
        let image_hash = hasher.hash_image(image);

        DigestDto {
            algorithm: format!("{:?}", alg ),
            value: image_hash.to_base64(),
            elapsed_time: now.elapsed().as_millis(),
        }
    }
}

impl ImageHashing for PHashImageHashing {


    fn digest<P: AsRef<Path>>(&self, path: P) -> Result<Vec<DigestDto>> {
        match image::open(&path) {
            Ok(image) => {
                Ok(vec![self.create_hash(&image,HashAlg::Mean ),
                // self.create_hash(&image,HashAlg::Blockhash ),
                // self.create_hash(&image,HashAlg::Gradient ),
                // self.create_hash(&image,HashAlg::VertGradient ),
                self.create_hash(&image,HashAlg::DoubleGradient )])
            }
            Err(err) => Err(anyhow!("{}", err)),
        }
    }

    fn distance(&self, hash1: &str, hash2: &str) -> Result<u32> {
        let first: Result<ImageHash, _> = ImageHash::from_base64(hash1);
        let second: Result<ImageHash, _> = ImageHash::from_base64(hash2);

        match (first, second) {
            (Ok(first_hash), Ok(second_hash)) => Ok(first_hash.dist(&second_hash)),
            _ => Err(anyhow!("Error decoding image hash for DoubleGradient")),
        }
    }
}
