use crate::domain::model::DigestDto;
use crate::domain::ports::incoming::Digester;
use anyhow::Result;
use ring::digest::{Context, SHA256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::Instant;

pub struct Sha256Digest {}

impl Digester for Sha256Digest {
    fn digest<P: AsRef<Path>>(&self, path: P) -> Result<DigestDto> {
        let now = Instant::now();
        let input = File::open(path)?;
        let mut reader = BufReader::new(input);
        let mut context = Context::new(&SHA256);
        let mut buffer = [0; 1024];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            context.update(&buffer[..count]);
        }
        let digest = context.finish();

        Ok(DigestDto {
            algorithm: "Sha256".to_string(),
            value: hex::encode(digest.as_ref()),
            elapsed_time: now.elapsed().as_millis(),
        })
    }
}
