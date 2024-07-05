use std::{convert::From, time::SystemTime};
use std::collections::HashMap;
use std::sync::Arc;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::prelude::{DateTime, Utc};

const PUZZLE_PREFIX: &str = "PUZZLE";
const USER_PREFIX: &str = "USER";
const STAMP_PREFIX: &str = "STAMP";
const DB_SEPARATOR: &str = "#";
const PUZZLE_PK_LENGTH: usize = 6;

fn rfc3339(st: &SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    dt.to_rfc3339() 
}

fn generate_string(length: usize) -> String {
    let mut rng = rand::thread_rng();

    // Generate alphanumeric characters
    let alphanumeric_string: String = Alphanumeric.sample_string(&mut rng, length);

    // Replace some characters with special characters
    let final_string: Vec<char> = alphanumeric_string.chars().collect();

    final_string.into_iter().collect::<String>().to_uppercase()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleDeserializer {
    pub puzzle_id: Option<String>,
    pub title: String,
    pub name: String,
    pub url: String,
    pub num_pieces: u16,
    pub completion_time: Option<u64>,
    pub users: Vec<UserDeserializer>,
    pub lat: f32,
    pub lng: f32,
    pub stamps: Vec<PuzzleStampDeserializer>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleSerializer {
    pub puzzle_id: String,
    pub title: String,
    pub name: String,
    pub url: String,
    pub num_pieces: u16,
    pub completion_time: Option<u64>,
    pub users: Vec<UserSerializer>,
    pub lat: f32,
    pub lng: f32,
    pub inserted: String,
    pub updated: String,
    pub stamps: Vec<PuzzleStampSerializer>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDeserializer {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSerializer {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub inserted: String,
    pub updated: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleStampDeserializer{
    pub users: Vec<UserDeserializer>,
    pub name: Option<String>, 
    pub missing_pieces: u16,
    pub completion_time: Option<u64>,
    pub urls: Vec<String>, // max six
    pub lat: f32,
    pub lng: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleStampSerializer{
    pub stamp_id: String,
    pub users: Vec<UserSerializer>,
    pub name: Option<String>, 
    pub missing_pieces: u16,
    pub completion_time: Option<u64>,
    pub urls: Vec<String>, // max six
    pub lat: f32,
    pub lng: f32,
    pub inserted: String,
    pub updated: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Puzzle {
    pub pk: String,
    pub sk: String,
    pub title: String,
    pub name: String,
    pub url: String,
    pub lat: f32,
    pub lng: f32,
    pub completion_time: Option<u64>,
    pub num_pieces: u16,
    pub inserted: String,
    pub updated: String,
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub pk: String,
    pub sk: String,
    pub email: String,
    pub name: String,   
    pub inserted: String,
    pub updated: String,
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleStamp{
    pub pk: String,
    pub sk: String,
    pub name: Option<String>, 
    pub missing_pieces: u16,
    pub completion_time: Option<u64>,
    pub urls: Vec<String>,
    pub lat: f32,
    pub lng: f32,
    pub inserted: String,
    pub updated: String,
}

pub fn from_database_hash(hash: &String) -> String {
     let parts: Vec<&str> = hash.split(DB_SEPARATOR).map(|s| s).collect();
     match parts.get(1) {
        Some(part) => part.to_string(),
        None => "".to_string(), // this should never happen
     }
}

pub fn to_database_hash(prefix: &str, id: &String) -> String {
    format!("{}#{}", prefix, id)
}

pub fn to_puzzle_pk(puzzle_id: &String) -> String {
    to_database_hash(PUZZLE_PREFIX, puzzle_id)
}

pub fn to_user_pk(puzzle_id: &String) -> String {
    to_database_hash(USER_PREFIX, puzzle_id)
}

impl From<&PuzzleDeserializer> for PuzzleSerializer {
    fn from(item: &PuzzleDeserializer) -> Self {
        PuzzleSerializer {
            puzzle_id: generate_string(PUZZLE_PK_LENGTH),
            title: item.title.clone(),
            name: item.name.clone(),
            url: item.url.clone(),
            num_pieces: item.num_pieces,
            completion_time: item.completion_time,
            users: user_deserializer_to_serializer(&item.users),
            lat: item.lat,
            lng: item.lng,
            stamps: puzzle_stamp_deserializer_to_serializer(&item.stamps),
            inserted: rfc3339(&SystemTime::now()),
            updated: rfc3339(&SystemTime::now()),

        }
    }
}


impl From<&PuzzleStampDeserializer> for PuzzleStampSerializer {
    fn from(item: &PuzzleStampDeserializer) -> Self {
        PuzzleStampSerializer {
            stamp_id: Uuid::new_v4().to_string(),
            users: user_deserializer_to_serializer(&item.users),
            name: item.name.clone(), 
            missing_pieces: item.missing_pieces,
            completion_time: item.completion_time,
            urls: item.urls.clone(),
            lat: item.lat,
            lng: item.lng,
            inserted: rfc3339(&SystemTime::now()),
            updated: rfc3339(&SystemTime::now()),
        }
    }
}

impl From<(Arc<PuzzleStamp>, &HashMap<String, Vec<Arc<User>>>)> for PuzzleStampSerializer {
    fn from((stamp, users): (Arc<PuzzleStamp>, &HashMap<String, Vec<Arc<User>>>)) -> Self {
        let stamp_id = from_database_hash(&stamp.sk); 
        let users = match users.get(&stamp.sk) {
            Some(u) => users_serializer_from_user(u),
            None => vec![],
        };
        PuzzleStampSerializer {
            stamp_id,
            users,
            name: stamp.name.clone(), 
            missing_pieces: stamp.missing_pieces,
            completion_time: stamp.completion_time,
            urls: stamp.urls.clone(),
            lat: stamp.lat,
            lng: stamp.lng,
            inserted: rfc3339(&SystemTime::now()),
            updated: rfc3339(&SystemTime::now()),
        }
    }
}


impl From<(&Arc<Puzzle>, Vec<PuzzleStampSerializer>, &Vec<Arc<User>>)> for PuzzleSerializer {
    fn from((puzzle, puzzle_stamps, users) : (&Arc<Puzzle>, Vec<PuzzleStampSerializer>, &Vec<Arc<User>>)) -> Self {
        PuzzleSerializer{
            puzzle_id: from_database_hash(&puzzle.pk),
            title: puzzle.title.clone(),
            name: puzzle.name.clone(),
            url: puzzle.url.clone(),
            num_pieces: puzzle.num_pieces,
            completion_time: puzzle.completion_time,
            users: users_serializer_from_user(users),
            stamps: puzzle_stamps,
            lat: puzzle.lat,
            lng: puzzle.lng,
            inserted: puzzle.inserted.clone(),
            updated: puzzle.updated.clone(),

        }
    }
}

impl From<&UserDeserializer> for UserSerializer {
    fn from(item: &UserDeserializer) -> Self {
        UserSerializer {
            user_id: Uuid::new_v4().to_string(),
            email: item.email.clone(),
            name: item.name.clone(), 
            inserted: rfc3339(&SystemTime::now()),
            updated: rfc3339(&SystemTime::now()),

        }
    }
}

impl From<&Arc<User>> for UserSerializer {
    fn from(item: &Arc<User>) -> Self {
        UserSerializer {
            user_id: Uuid::new_v4().to_string(),
            email: item.email.clone(),
            name: item.name.clone(), 
            inserted: rfc3339(&SystemTime::now()),
            updated: rfc3339(&SystemTime::now()),

        }
    }
}

impl From<(&PuzzleSerializer, &UserSerializer)> for User {
    fn from((puzzle, item) : (&PuzzleSerializer, &UserSerializer)) -> Self {
        User {
            pk: to_database_hash(PUZZLE_PREFIX, &puzzle.puzzle_id), 
            sk: to_database_hash(USER_PREFIX, &item.user_id), 
            email: item.email.clone(), 
            name: item.name.clone(), 
            inserted: item.inserted.clone(), 
            updated: item.updated.clone(), 
        }
    }
}

impl From<(&PuzzleStampSerializer, &UserSerializer)> for User {
    fn from((stamp, item) : (&PuzzleStampSerializer, &UserSerializer)) -> Self {
        User {
            pk: to_database_hash(STAMP_PREFIX, &stamp.stamp_id), 
            sk: to_database_hash(USER_PREFIX, &item.user_id), 
            email: item.email.clone(), 
            name: item.name.clone(), 
            inserted: item.inserted.clone(), 
            updated: item.updated.clone(), 
        }
    }
}

impl From<&PuzzleSerializer> for Puzzle {
    fn from(item: &PuzzleSerializer) -> Self {
        Puzzle { 
            pk: to_database_hash(PUZZLE_PREFIX, &item.puzzle_id), 
            sk: to_database_hash(PUZZLE_PREFIX, &item.puzzle_id), 
            title: item.title.clone(),
            url: item.url.clone(),
            lat: item.lat,
            lng: item.lng,
            name: item.name.clone(),
            completion_time: item.completion_time,
            num_pieces: item.num_pieces, 
            inserted: item.inserted.clone(), 
            updated: item.updated.clone(),
        }
    }
}

impl From<(&String, &PuzzleStampSerializer)> for  PuzzleStamp {
    fn from((puzzle_id, item) : (&String , &PuzzleStampSerializer)) -> Self {
        PuzzleStamp { 
            pk: to_database_hash(PUZZLE_PREFIX, puzzle_id), 
            sk: to_database_hash(STAMP_PREFIX, &item.stamp_id), 
            name: item.name.clone(), 
            missing_pieces: item.missing_pieces,
            completion_time: item.completion_time,
            urls: item.urls.clone(),
            lat: item.lat, 
            lng: item.lng, 
            inserted: item.inserted.clone(), 
            updated: item.updated.clone(),
        }
    }
}

fn users_serializer_from_user(items: &Vec<Arc<User>>) -> Vec<UserSerializer> {
    items.into_iter().map(|u| u.into()).collect()
}

fn user_deserializer_to_serializer(items: &Vec<UserDeserializer>) -> Vec<UserSerializer> {
    items.into_iter().map(|item| item.into()).collect()
}

fn puzzle_stamp_deserializer_to_serializer(items: &Vec<PuzzleStampDeserializer>) -> Vec<PuzzleStampSerializer> {
    items.into_iter().map(|item| item.into()).collect()

}
