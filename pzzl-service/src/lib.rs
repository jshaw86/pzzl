pub mod types;
mod util;
use serde_dynamo::to_item;
use std::sync::Arc;
use std::collections::HashMap;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_dynamodb::types::{AttributeValue, TransactWriteItem, Put};
use anyhow::{Error, Result};
use crate::types::{User, Puzzle, PuzzleDeserializer, PuzzleSerializer, PuzzleStampDeserializer, PuzzleStampSerializer, PuzzleStamp};
use core::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct PzzlService {
   pub dynamo_client: Arc<DynamoClient>,
   pub s3_client: Arc<S3Client>
}

const MAX_PUZZLE_INSERT_TRYS: usize = 10;

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

    pub async fn add_stamps(&self, puzzle_id: String, stamps: Vec<PuzzleStampDeserializer>) -> Result<PuzzleSerializer> {
        let mut write_items = vec![];
        for stamp in stamps {
            let stamp_response: PuzzleStampSerializer = (&stamp).into();
            let db_stamp = to_item(&PuzzleStamp::from((&puzzle_id, &stamp_response)))?;

            write_items.push(
                TransactWriteItem::builder()
                .put(Put::builder().table_name("puzzles_users")
                     .set_item(Some(db_stamp))
                     .build()?
                    ).build());

            for user in &stamp_response.users {
                let db_user = to_item(&User::from((&stamp_response, user)))?;
                write_items.push(
                    TransactWriteItem::builder()
                    .put(Put::builder().table_name("puzzles_users")
                         .set_item(Some(db_user))
                         .build()?
                        ).build());

            }

        }

        let _ = self.dynamo_client
            .transact_write_items()
            .set_transact_items(Some(write_items))
            .send().await?;

       self.get_puzzle(puzzle_id).await

    }

    // Function to create a new user
    pub async fn insert_puzzle(&self, puzzle: PuzzleDeserializer) -> Result<PuzzleSerializer> {  
        loop {
            let puzzle_response: PuzzleSerializer = (&puzzle).into(); 
            let mut write_items: Vec<TransactWriteItem> = vec![];
            let puzzle_id = &puzzle_response.puzzle_id;

            let spuzzle = to_item(&Puzzle::from(&puzzle_response))?;

            write_items.push(
                TransactWriteItem::builder()
                .put(
                    Put::builder().table_name("puzzles_users")
                    .condition_expression("pk <> :pk")
                    .expression_attribute_values(":pk", AttributeValue::S(types::to_puzzle_pk(&puzzle_id)))
                    .set_item(Some(spuzzle)).build()?
                    ).build()
                );

            for stamp_response in &puzzle_response.stamps {
                let db_stamp = to_item(&PuzzleStamp::from((puzzle_id, stamp_response)))?;

                write_items.push(
                    TransactWriteItem::builder()
                    .put(Put::builder().table_name("puzzles_users")
                         .set_item(Some(db_stamp))
                         .build()?
                        ).build());

                for user in &stamp_response.users {
                    let db_user = to_item(&User::from((stamp_response, user)))?;
                    write_items.push(
                        TransactWriteItem::builder()
                        .put(Put::builder().table_name("puzzles_users")
                             .set_item(Some(db_user))
                             .build()?
                            ).build());

                }

            }

            for user in &puzzle_response.users {
                let db_user = to_item(&User::from((&puzzle_response, user)))?;
                write_items.push(
                    TransactWriteItem::builder()
                    .put(Put::builder().table_name("puzzles_users")
                         .set_item(Some(db_user))
                         .build()?
                        ).build());

            }

            let mut insert_trys = 0;
            let transact_write_item = self.dynamo_client
                .transact_write_items()
                .set_transact_items(Some(write_items))
                .send().await;

            if let Ok(_) = transact_write_item {
                return self.get_puzzle(puzzle_id.to_string()).await;
            }

            insert_trys += 1;

            if insert_trys > MAX_PUZZLE_INSERT_TRYS {
                return Err(Error::msg("could not find a unique puzzle id, try again later")); 
            }

        }
       
    }
    // Function to fetch a user's profile
    pub async fn get_puzzle(&self, puzzle_id: String) -> Result<PuzzleSerializer> {
        let (puzzle, puzzle_stamps, puzzle_users): (Arc<Puzzle>, Vec<Arc<PuzzleStamp>>, Vec<Arc<User>>) = self.get_puzzle_and_stamps(&puzzle_id).await?;

        let stamp_to_users = self.get_users_by_stamp_id(&puzzle_stamps).await?;

        let puzzle_stamp_serializers: Vec<PuzzleStampSerializer> = puzzle_stamps.into_iter().map(|puzzle_stamp|  PuzzleStampSerializer::from((puzzle_stamp, &stamp_to_users))).collect();

        Ok(PuzzleSerializer::from((&puzzle, puzzle_stamp_serializers, &puzzle_users)))

    }

    async fn get_puzzle_and_stamps(&self, puzzle_id: &String) -> Result<(Arc<Puzzle>, Vec<Arc<PuzzleStamp>>, Vec<Arc<User>>)>{
        let db_attribute = AttributeValue::S(types::to_puzzle_pk(puzzle_id));
        let batch_result = self.dynamo_client 
            .execute_statement()
            .statement(format!(
                    r#"SELECT * FROM "{}" WHERE "{}" = ?"#,
                    "puzzles_users", "pk" 
                    ))
            .set_parameters(Some(vec![db_attribute.clone()]))
            .send()
            .await?;
        
            match util::parse_puzzle_stamps_users(&batch_result)?{
                (Some(puzzle), stamps, users) => Ok((puzzle, stamps, users)),
                (None, _, _) => Err(Error::msg("No puzzle found")),
            }
    }

    async fn get_users_by_stamp_id(&self, stamps: &Vec<Arc<PuzzleStamp>>) -> Result<HashMap<String, Vec<Arc<User>>>> {
        let stamp_ids: Vec<AttributeValue> = stamps.into_iter().map(|stamp| AttributeValue::S(stamp.sk.to_string())).collect();

        let batch_result = self.dynamo_client 
            .execute_statement()
            .statement(format!(
                    r#"SELECT * FROM "{}" WHERE "{}" IN [?]"#,
                    "puzzles_users", "pk", 
                    ))
            .set_parameters(Some(stamp_ids))    
            .send()
            .await?;

        let stamp_users: Vec<Arc<User>> = util::parse_users(&batch_result)?;

        let result = util::stamp_user_mapping(&stamps, &stamp_users);

        Ok(result)

    }
}
