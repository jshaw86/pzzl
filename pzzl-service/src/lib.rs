pub mod types;
pub mod db;
mod util;
use std::sync::Arc;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::presigning::PresigningConfig;
use anyhow::Result;
use crate::types::{PuzzleDeserializer, PuzzleSerializer, PuzzleStampDeserializer, MediaSerializer};
use core::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct PzzlService {
   pub database: db::PzzlDatabase,
   pub s3_client: Arc<S3Client>
}


impl PzzlService {

    pub async fn get_media_url(&self, prefix: String, bucket_name: String, content_type: String) -> Result<MediaSerializer> {
        let expiration = Duration::from_secs(15 * 60);
        let object_key = format!("{}-{}", prefix, Uuid::new_v4().to_string()); 

         let presigned_req = self.s3_client
        .put_object()
        .bucket(bucket_name)
        .key(object_key)
        .content_type(content_type) // Set the desired content type
        .presigned(
            PresigningConfig::builder()
                .expires_in(expiration) // Expiration time for the presigned URL
                .build()?
        )
        .await?;

        Ok(MediaSerializer {
            uri: presigned_req.uri().to_string(),
            method: presigned_req.method().to_string(),
            headers: presigned_req.headers().map(|(k, v)| (k.to_string(), v.to_string())).collect()
        })
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
