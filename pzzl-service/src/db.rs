use std::sync::Arc;
use std::{convert::From, time::SystemTime};
use std::collections::HashMap;
use std::borrow::Cow;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_dynamodb::types::{AttributeValue, TransactWriteItem, Put};
use serde_dynamo::{from_item, to_item};
use anyhow::{Result, Error};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::types::{PuzzleSerializer, PuzzleStampSerializer, PuzzleStampDeserializer, UserSerializer, PuzzleDeserializer};
use crate::util;

const PUZZLE_PREFIX: &str = "PUZZLE";
const USER_PREFIX: &str = "USER";
const STAMP_PREFIX: &str = "STAMP";
const DB_SEPARATOR: &str = "#";
const MAX_PUZZLE_INSERT_TRYS: usize = 10;


#[derive(Debug, Clone)]
pub struct PzzlDatabase {
   pub client: Arc<DynamoClient>,

}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Puzzle<'response> {
    pub pk: Cow<'response, str>,
    pub sk: Cow<'response, str>,
    pub title: Cow<'response, str>,
    pub name: Cow<'response, str>,
    pub url: Cow<'response, str>,
    pub lat: f32,
    pub lng: f32,
    pub completion_time: Option<u64>,
    pub num_pieces: u16,
    pub inserted: Cow<'response, str>,
    pub updated: Cow<'response, str>,
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
struct User<'response> {
    pub pk: Cow<'response, str>,
    pub sk: Cow<'response, str>,
    pub email: Cow<'response, str>,
    pub name: Cow<'response, str>,   
    pub inserted: Cow<'response, str>,
    pub updated: Cow<'response, str>,
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PuzzleStamp <'response>{
    pub pk: Cow<'response, str>,
    pub sk: Cow<'response, str>,
    pub name: Option<Cow<'response, str>>, 
    pub missing_pieces: u16,
    pub completion_time: Option<u64>,
    pub urls: Vec<Cow<'response, str>>,
    pub lat: f32,
    pub lng: f32,
    pub inserted: Cow<'response, str>,
    pub updated: Cow<'response, str>,
}

impl PzzlDatabase {

    pub async fn insert_puzzle(&self, puzzle: &PuzzleDeserializer) -> Result<PuzzleSerializer> {
        loop {
            let puzzle_response: PuzzleSerializer = puzzle.into(); 
            let mut write_items: Vec<TransactWriteItem> = vec![];
            let puzzle_id = puzzle_response.puzzle_id.as_str();

            let spuzzle = to_item(&Puzzle::from(&puzzle_response))?;

            write_items.push(
                TransactWriteItem::builder()
                .put(
                    Put::builder().table_name("puzzles_users")
                    .condition_expression("pk <> :pk")
                    .expression_attribute_values(":pk", AttributeValue::S(to_database_hash(PUZZLE_PREFIX, puzzle_id).to_string()))
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
            let transact_write_item = self.client
                .transact_write_items()
                .set_transact_items(Some(write_items))
                .send().await;

            if let Ok(_) = transact_write_item {
                return self.get_puzzle(puzzle_id).await;
            }

            insert_trys += 1;

            if insert_trys > MAX_PUZZLE_INSERT_TRYS {
                return Err(Error::msg("could not find a unique puzzle id, try again later")); 
            }

        }

    }

    pub async fn add_stamps(&self, puzzle_id: &str, stamps: Vec<&PuzzleStampDeserializer>) -> Result<PuzzleSerializer> {
        let mut write_items = vec![];
        for stamp in stamps {
            let stamp_response: PuzzleStampSerializer = stamp.into();
            let db_stamp = to_item(&PuzzleStamp::from((puzzle_id, &stamp_response)))?;

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

        let _ = self.client
            .transact_write_items()
            .set_transact_items(Some(write_items))
            .send().await?;

       self.get_puzzle(&puzzle_id).await


    }

    pub async fn get_puzzle(&self, puzzle_id: &str) -> Result<PuzzleSerializer> {
        let db_puzzle_stamps = self.get_puzzle_and_stamps(puzzle_id).await?;
        let (possible_puzzle, puzzle_stamps, puzzle_users) = PzzlDatabase::parse_puzzle_stamps_users(db_puzzle_stamps)?;

        if let Some(puzzle) = possible_puzzle {
            let db_stamp_users = self.get_users_by_stamp_id(&puzzle_stamps).await?;
            let stamp_users = PzzlDatabase::parse_users(db_stamp_users)?;
            let stamp_to_users: HashMap<String, Vec<User>> = PzzlDatabase::stamp_user_mapping(&puzzle_stamps, stamp_users);

            let mut puzzle_stamp_serializers: Vec<PuzzleStampSerializer> = vec![];

            for puzzle_stamp in puzzle_stamps {
                let users: Vec<UserSerializer> = match stamp_to_users.get(&puzzle_stamp.sk.to_string()) {
                    Some(users) => users.iter().cloned().map(UserSerializer::from).collect(), //users.iter().cloned().map(UserSerializer::from).collect(),
                    None => vec![],
                };
                let puzzle_stamp_serializer = PuzzleStampSerializer::from((puzzle_stamp, users));
                puzzle_stamp_serializers.push(puzzle_stamp_serializer);
            }
            return Ok(PuzzleSerializer::from((puzzle, puzzle_stamp_serializers, puzzle_users.iter().cloned().map(UserSerializer::from).collect())));
        }

        Err(Error::msg("No puzzle found"))
    }

    async fn get_puzzle_and_stamps<'response>(&self, puzzle_id: &'response str) -> Result<Vec<HashMap<String, AttributeValue>>>{
        let db_attribute = AttributeValue::S(to_database_hash(PUZZLE_PREFIX, puzzle_id).to_string());
        let batch_result = self.client 
            .execute_statement()
            .statement(format!(
                    r#"SELECT * FROM "{}" WHERE "{}" = ?"#,
                    "puzzles_users", "pk" 
                    ))
            .set_parameters(Some(vec![db_attribute.clone()]))
            .send()
            .await?;

       if let Some(items) = batch_result.items {
           return Ok(items)
       }

       Err(Error::msg("No puzzle Found"))
            
    }

    async fn get_users_by_stamp_id<'response>(&self, stamps: &Vec<PuzzleStamp<'response>>) -> Result<Vec<HashMap<String, AttributeValue>>> {
        let stamp_ids: Vec<AttributeValue> = stamps.into_iter().map(|stamp| AttributeValue::S(stamp.sk.to_string())).collect();

        let batch_result = self.client 
            .execute_statement()
            .statement(format!(
                    r#"SELECT * FROM "{}" WHERE "{}" IN [?]"#,
                    "puzzles_users", "pk", 
                    ))
            .set_parameters(Some(stamp_ids))    
            .send()
            .await?;

        if let Some(items) = batch_result.items {
           return Ok(items)
       }

       Err(Error::msg("No users found on stamp"))

    }

    fn stamp_user_mapping<'response>(stamps: &Vec<PuzzleStamp<'response>>, stamps_users: Vec<User<'response>>) -> HashMap<String, Vec<User<'response>>> {
        let mut users_stamps: HashMap<String, Vec<User>> = HashMap::new();

        for stamp in stamps {
            for user in &stamps_users {
                let entry = users_stamps.entry(stamp.sk.to_string()).or_insert_with(Vec::new);
                entry.push(user.clone());
            }
        }

        users_stamps
    }

    fn parse_puzzle_stamps_users<'response>(items: Vec<HashMap<String, AttributeValue>>) -> Result<(Option<Puzzle<'response>>, Vec<PuzzleStamp<'response>>, Vec<User<'response>>)> {
        let mut puzzle_stamps: Vec<PuzzleStamp> = vec![];
        let mut puzzle_users: Vec<User> = vec![];
        let mut puzzle: Option<Puzzle> = None;

        for item in items {
            let puzzle_stamp_result: Result<PuzzleStamp, serde_dynamo::Error> = from_item(item.clone());
            if let Ok(stamp) = puzzle_stamp_result {
                puzzle_stamps.push(stamp);
                continue;
            }

            let user_result: Result<User, serde_dynamo::Error> = from_item(item.clone());
            if let Ok(user) = user_result {
                puzzle_users.push(user);
                continue
            }

            let puzzle_result: Result<Puzzle, serde_dynamo::Error> = from_item(item.clone());
            if let Ok(pzzl) =  puzzle_result {
                puzzle = Some(pzzl);
                continue
            }

            return Err(Error::msg(format!("unable to parse item {:?}", item)));
        }

        return Ok((puzzle, puzzle_stamps, puzzle_users));

    }

    fn parse_users<'response>(items: Vec<HashMap<String, AttributeValue>>) -> Result<Vec<User<'response>>> {
        let mut users: Vec<User> = vec![];
        for item in items {
            let user: User = from_item(item.clone())?;

            users.push(user);

        }

        Ok(users)
    }

}

impl<'response> From<&'response PuzzleSerializer> for Puzzle<'response> {
    fn from(item: &'response PuzzleSerializer) -> Self {
        Puzzle { 
            pk: to_database_hash(PUZZLE_PREFIX, &item.puzzle_id), 
            sk: to_database_hash(PUZZLE_PREFIX, &item.puzzle_id), 
            title: Cow::Borrowed(&item.title),
            url: Cow::Borrowed(&item.url),
            lat: item.lat,
            lng: item.lng,
            name: Cow::Borrowed(&item.name),
            completion_time: item.completion_time,
            num_pieces: item.num_pieces, 
            inserted: Cow::Borrowed(&item.inserted), 
            updated: Cow::Borrowed(&item.updated),
        }
    }
}

impl<'response> From<(&'response str, &'response PuzzleStampSerializer)> for  PuzzleStamp<'response> {
    fn from((puzzle_id, item) : (&str , &'response PuzzleStampSerializer)) -> Self {
        PuzzleStamp { 
            pk: to_database_hash(PUZZLE_PREFIX, puzzle_id), 
            sk: to_database_hash(STAMP_PREFIX, &item.stamp_id), 
            name: match &item.name {
                Some(name) => Some(Cow::Borrowed(name)), 
                None => None 
            },
            missing_pieces: item.missing_pieces,
            completion_time: item.completion_time,
            urls: item.urls.iter().map(|url| Cow::Borrowed(&**url)).collect(),
            lat: item.lat, 
            lng: item.lng, 
            inserted: Cow::Borrowed(&item.inserted), 
            updated: Cow::Borrowed(&item.updated),
        }
    }
}

impl<'response> From<(&'response PuzzleSerializer, &'response UserSerializer)> for User<'response> {
    fn from((puzzle, item) : (&'response PuzzleSerializer, &'response UserSerializer)) -> Self {
        User {
            pk: to_database_hash(PUZZLE_PREFIX, &puzzle.puzzle_id), 
            sk: to_database_hash(USER_PREFIX, &item.user_id), 
            email: Cow::Borrowed(&item.email), 
            name: Cow::Borrowed(&item.name), 
            inserted: Cow::Borrowed(&item.inserted), 
            updated: Cow::Borrowed(&item.updated), 
        }
    }
}

impl<'response> From<(&'response PuzzleStampSerializer, &'response UserSerializer)> for User<'response> {
    fn from((stamp, item) : (&'response PuzzleStampSerializer, &'response UserSerializer)) -> Self {
        User {
            pk: to_database_hash(STAMP_PREFIX, &stamp.stamp_id), 
            sk: to_database_hash(USER_PREFIX, &item.user_id), 
            email: Cow::Borrowed(&item.email), 
            name: Cow::Borrowed(&item.name), 
            inserted: Cow::Borrowed(&item.inserted), 
            updated: Cow::Borrowed(&item.updated), 
        }
    }
}

      

// intentionally owned objects here because we need to copy ownership away from the references
// created in get_puzzle for PuzzleStamp and the HashMap of Users
impl<'response> From<(PuzzleStamp<'response>, Vec<UserSerializer>)> for PuzzleStampSerializer {
    fn from((stamp, users): (PuzzleStamp<'response>, Vec<UserSerializer>)) -> Self {
        PuzzleStampSerializer {
            stamp_id: from_database_hash(&stamp.sk.to_string()).into(),
            users,
            name: stamp.name.map(String::from), 
            missing_pieces: stamp.missing_pieces,
            completion_time: stamp.completion_time,
            urls: stamp.urls.into_iter().map(|url| url.to_string()).collect(),
            lat: stamp.lat,
            lng: stamp.lng,
            inserted: util::rfc3339(&SystemTime::now()).into(),
            updated: util::rfc3339(&SystemTime::now()).into(),
        }
    }
}

// intentionally owned objects here because we need to copy ownership away from the references
// created in get_puzzle for Puzzle, PuzzleStampSerializer and Users 
impl<'response> From<(Puzzle<'response>, Vec<PuzzleStampSerializer>, Vec<UserSerializer>)> for PuzzleSerializer {
    fn from((puzzle, puzzle_stamps, users) : (Puzzle<'response>, Vec<PuzzleStampSerializer>, Vec<UserSerializer>)) -> Self {
        PuzzleSerializer{
            puzzle_id: from_database_hash(&puzzle.pk.to_string()).into(),
            title: puzzle.title.to_string(),
            name: puzzle.name.to_string(),
            url: puzzle.url.to_string(),
            num_pieces: puzzle.num_pieces,
            completion_time: puzzle.completion_time,
            users,
            stamps: puzzle_stamps,
            lat: puzzle.lat,
            lng: puzzle.lng,
            inserted: puzzle.inserted.to_string(),
            updated: puzzle.updated.to_string(),

        }
    }
}

impl<'response> From<User<'response>> for UserSerializer {
    fn from(item: User<'response>) -> Self {
        UserSerializer {
            user_id: Uuid::new_v4().to_string().into(),
            email: item.email.to_string(),
            name: item.name.to_string(), 
            inserted: util::rfc3339(&SystemTime::now()).into(),
            updated: util::rfc3339(&SystemTime::now()).into(),

        }
    }
}

pub fn from_database_hash(hash: &String) -> String {
     let parts: Vec<&str> = hash.split(DB_SEPARATOR).map(|s| s).collect();
     match parts.get(1) {
        Some(part) => part.to_string(),
        None => "".to_string(), // this should never happen
     }
}

pub fn to_database_hash<'response>(prefix: &str, id: &str) -> Cow<'response, str> {
    format!("{}#{}", prefix, id).into()
}

