pub mod types;
pub mod db;
mod util;
use std::sync::Arc;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::presigning::PresigningConfig;
use anyhow::Result;
use crate::types::{PuzzleDeserializer, PuzzleSerializer, PuzzleStampDeserializer};
use core::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct PzzlService {
   pub database: db::PzzlDatabase,
   pub s3_client: Arc<S3Client>
}


impl PzzlService {

    pub async fn get_media_url(&self, prefix: String, bucket_name: String) -> Result<String> {
        let expiration = Duration::from_secs(15 * 60);
        let object_key = format!("{}-{}", prefix, Uuid::new_v4().to_string()); 

        // Create a PutObjectRequest
        let req = self.s3_client
            .put_object()
            .bucket(bucket_name)
            .key(object_key)
            .body(ByteStream::from_static(b""));

        // Configure the presigned request with expiration time
        let presigning_config = PresigningConfig::expires_in(expiration)?;

        // Generate the presigned URL
        let presigned_req = req.presigned(presigning_config).await?;
        return Ok(presigned_req.uri().to_string());
    }

  

    // Function to create a new user
    pub async fn insert_puzzle(&self, puzzle: &PuzzleDeserializer) -> Result<PuzzleSerializer> {  
        Ok(self.database.insert_puzzle(puzzle).await?)        
    }

    pub async fn add_stamps(&self, puzzle_id: &str, stamps: Vec<&PuzzleStampDeserializer>) -> Result<PuzzleSerializer> {
        Ok(self.database.add_stamps(puzzle_id, stamps).await?)
    }
    // Function to fetch a user's profile
    pub async fn get_puzzle(&self, puzzle_id: &str) -> Result<PuzzleSerializer> {
        Ok(self.database.get_puzzle(puzzle_id).await?)
    }
}
