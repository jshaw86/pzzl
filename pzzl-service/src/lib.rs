pub mod types;
mod util;
use serde_dynamo::to_item;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::SystemTime;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, TransactWriteItem, Put};
use anyhow::{Error, Result};
use crate::types::{User, Puzzle, PuzzleSerializer, PuzzleUserSerializer, PuzzleUser, FillDates};
use core::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct PzzlService {
   pub dynamo_client: Arc<DynamoClient>,
   pub s3_client: Arc<S3Client>
}

const PUZZLE_PK_LENGTH: usize = 6;
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

    pub async fn add_stamps(&self, puzzle_id: String, puzzle_users: Vec<PuzzleUserSerializer>) -> Result<PuzzleSerializer> {
        let mut write_items = vec![];
        for puzzle_user in puzzle_users {
            let mutable_puzzle_user = util::fill_owned(&util::fill_user_id(&puzzle_user.fill_dates(Some(SystemTime::now()))), false);
            let suser = to_item(&User::from(&mutable_puzzle_user))?;
            let puzzle_user_db = types::make_puzzle_user(&mutable_puzzle_user, &puzzle_id);
            let suser_puzzle = to_item(&puzzle_user_db)?;
            write_items.push(
                TransactWriteItem::builder()
                .put(Put::builder().table_name("puzzles_users")
                     .set_item(Some(suser_puzzle))
                     .build()?
                    ).build());

            write_items.push(
                TransactWriteItem::builder()
                .put(Put::builder().table_name("puzzles_users")
                     .set_item(Some(suser))
                     .build()?
                    ).build());

        }

        let _ = self.dynamo_client
        .transact_write_items()
        .set_transact_items(Some(write_items))
                    .send().await?;

        Ok(self.get_puzzle(puzzle_id).await?)

    }

    // Function to create a new user
    pub async fn insert_puzzle(&self, puzzle: PuzzleSerializer) -> Result<PuzzleSerializer> {  
        let mut mutable_puzzle = puzzle.fill_dates(Some(SystemTime::now()));

        loop {
            let mut transact_write_requests: Vec<TransactWriteItem> = vec![];
            let puzzle_id = util::generate_string(PUZZLE_PK_LENGTH);
            mutable_puzzle.puzzle_id = Some(puzzle_id.clone());

            let spuzzle = to_item(&Puzzle::from(&mutable_puzzle))?;

            transact_write_requests.push(
                TransactWriteItem::builder()
                .put(
                    Put::builder().table_name("puzzles_users")
                    .condition_expression("pk <> :pk")
                    .expression_attribute_values(":pk", AttributeValue::S(types::to_puzzle_pk(&puzzle_id)))
                    .set_item(Some(spuzzle)).build()?
                    ).build()
                );

            for puzzle_user in &mutable_puzzle.stamps {
                let mutable_puzzle_user = util::fill_owned(&util::fill_user_id(&puzzle_user), true); 
                let suser = to_item(&User::from(&mutable_puzzle_user))?;

                transact_write_requests.push(
                    TransactWriteItem::builder()
                    .put(Put::builder().table_name("puzzles_users")
                         .set_item(Some(suser))
                         .build()?).build());

                let user_puzzle = types::make_puzzle_user(&mutable_puzzle_user, &puzzle_id);
                let suser_puzzle = to_item(&user_puzzle)?;

                transact_write_requests.push(
                    TransactWriteItem::builder()
                    .put(Put::builder().table_name("puzzles_users")
                         .set_item(Some(suser_puzzle))
                         .build()?).build());

            }

            let mut insert_trys = 0;
            let transact_write_item = self.dynamo_client
                .transact_write_items()
                .set_transact_items(Some(transact_write_requests.clone()))
                .send().await;

            if let Ok(_) = transact_write_item {
                return Ok(self.get_puzzle(puzzle_id).await?);
            }

            insert_trys += 1;

            if insert_trys > MAX_PUZZLE_INSERT_TRYS {
                return Err(Error::msg("could not find a unique puzzle id, try again later")); 
            }

        }

    }
    // Function to fetch a user's profile
    pub async fn get_puzzle(&self, puzzle_id: String) -> Result<PuzzleSerializer> {
        let puzzle_users_result = self.get_puzzle_and_puzzle_users(&puzzle_id).await;

        if let Err(err) = puzzle_users_result {
            return Err(err);
        }

        let (puzzle, puzzle_users) = puzzle_users_result.unwrap();

        let user_responses = self.get_users(&puzzle_users).await;

        if let Err(err) = user_responses {
            return Err(err);
        }

        let users: HashMap<String, User> = user_responses.unwrap().into_iter().map(|u| (u.pk.clone(), u)).collect();

        let puzzle_users_response: Vec<PuzzleUserSerializer> = types::make_puzzle_user_serializers(&users, &puzzle_users); 

        let mut puzzle_response: PuzzleSerializer = PuzzleSerializer::from(&puzzle);
        puzzle_response.stamps = puzzle_users_response;
        Ok(puzzle_response)

    }

    async fn get_puzzle_and_puzzle_users(&self, puzzle_id: &String) -> Result<(Puzzle, Vec<PuzzleUser>)>{
        let batch_result = self.dynamo_client 
            .execute_statement()
            .statement(format!(
                    r#"SELECT * FROM "{}" WHERE "{}" = ?"#,
                    "puzzles_users", "pk" 
                    ))
            .set_parameters(Some(vec![AttributeValue::S(types::to_puzzle_pk(puzzle_id))]))
            .send()
            .await?;

        Ok(util::parse_puzzle_puzzle_users(&batch_result)?)

    }

    async fn get_users(&self, puzzle_users: &Vec<PuzzleUser>) -> Result<Vec<User>> {
        let db_user_pks: Vec<HashMap<String, AttributeValue>> = puzzle_users.into_iter()
            .map(|u| HashMap::from([
                                   ("pk".to_string(), AttributeValue::S(u.sk.clone())),
                                   ("sk".to_string(), AttributeValue::S(u.sk.clone()))
            ])).collect();

        let users_query = KeysAndAttributes::builder()
            .set_keys(Some(db_user_pks))
            .build()?;

        let items = self.dynamo_client.batch_get_item()
            .request_items("puzzles_users",users_query)
            .send()
            .await?;

        return util::parse_users(&items);

    }
}
