use std::{convert::From, time::SystemTime};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::prelude::{DateTime, Utc};

const PUZZLE_PREFIX: &str = "PUZZLE";
//const PUZZLE_USER_PREFIX: &str = "PUZZLEUSER";
const USER_PREFIX: &str = "USER";
const DB_SEPARATOR: &str = "#";

pub trait FillDates {
    fn fill_dates(&self, d: Option<SystemTime>) -> Self;
}

fn rfc3339(st: &SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    dt.to_rfc3339() 
    
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleSerializer {
    pub puzzle_id: Option<String>,
    pub title: String,
    pub media: String,
    pub num_pieces: u16,
    pub inserted: Option<String>,
    pub updated: Option<String>,
    pub stamps: Vec<PuzzleUserSerializer>,
}

impl FillDates for PuzzleSerializer {
   fn fill_dates(&self, inserted: Option<SystemTime>) -> Self {
        let mut obj = self.clone();
        obj.inserted = match inserted {
            Some(date) => Some(rfc3339(&date)),
            None => obj.inserted 
        };
        obj.updated = Some(rfc3339(&SystemTime::now()));
        obj.stamps = obj.stamps.into_iter().map(|pu: PuzzleUserSerializer| pu.fill_dates(inserted)).collect();
        return obj;
   } 

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSerializer {
    pub user_id: Option<String>,
    pub email: String,
    pub name: String,
    pub owned: bool,
    pub inserted: Option<String>,
    pub updated: Option<String>,
}

impl FillDates for UserSerializer {
    fn fill_dates(&self, inserted: Option<SystemTime>) -> Self {
        let mut obj = self.clone();
        obj.inserted = match inserted {
            Some(date) => Some(rfc3339(&date)),
            None => obj.inserted 
        };
        
        obj.updated = Some(rfc3339(&SystemTime::now()));
        return obj;
    } 

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleUserSerializer{
    pub user: UserSerializer,
    pub name: Option<String>, 
    pub missing_pieces: u16,
    pub puzzlers: u16,
    pub completion_time: Option<u64>,
    pub media: Option<String>,
    pub lat: f32,
    pub lng: f32,
    pub inserted: Option<String>,
    pub updated: Option<String>,
}

impl FillDates for PuzzleUserSerializer {
    fn fill_dates(&self, inserted: Option<SystemTime>) -> Self {
        let mut obj = self.clone();
        obj.inserted = match inserted {
            Some(date) => Some(rfc3339(&date)),
            None => obj.inserted 
        };
        obj.updated = Some(rfc3339(&SystemTime::now()));

        obj.user = obj.user.fill_dates(inserted);
        return obj;
    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Puzzle {
    pub pk: String,
    pub sk: String,
    pub title: String,
    pub media: String,
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
    pub owned: bool,
    pub inserted: String,
    pub updated: String,
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleUser{
    pub pk: String,
    pub sk: String,
    pub name: Option<String>, 
    pub missing_pieces: u16,
    pub puzzlers: u16,
    pub completion_time: Option<u64>,
    pub media: Option<String>,
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

impl From<(&String, &PuzzleUserSerializer)> for PuzzleUser {
    fn from((puzzle_id, puzzle_user): (&String, &PuzzleUserSerializer)) -> Self {
        PuzzleUser {
            pk: to_database_hash(PUZZLE_PREFIX, puzzle_id),
            sk: to_database_hash(USER_PREFIX, &puzzle_user.user.user_id.clone().unwrap()),
            name: puzzle_user.name.clone(), 
            missing_pieces: puzzle_user.missing_pieces,
            puzzlers: puzzle_user.puzzlers,
            completion_time: puzzle_user.completion_time,
            media: puzzle_user.media.clone(),
            lat: puzzle_user.lat,
            lng: puzzle_user.lng,
            inserted: puzzle_user.inserted.clone().unwrap(),
            updated: puzzle_user.updated.clone().unwrap(),
        }
    }
}

pub fn make_puzzle_user(puzzle_user: &PuzzleUserSerializer, puzzle_id: &String) -> PuzzleUser {
    PuzzleUser {
        pk: to_database_hash(PUZZLE_PREFIX, &puzzle_id),
        sk: to_database_hash(USER_PREFIX, &puzzle_user.user.user_id.clone().unwrap()),
        name: puzzle_user.name.clone(), 
        missing_pieces: puzzle_user.missing_pieces,
        puzzlers: puzzle_user.puzzlers,
        completion_time: puzzle_user.completion_time,
        media: puzzle_user.media.clone(),
        lat: puzzle_user.lat,
        lng: puzzle_user.lng,
        inserted: puzzle_user.inserted.clone().unwrap(),
        updated: puzzle_user.updated.clone().unwrap(),
    }
}

pub fn make_puzzle_user_serializers(users: &HashMap<String,User>, puzzle_users: &Vec<PuzzleUser>) -> Vec<PuzzleUserSerializer> {
    let mut puzzle_user_serializers: Vec<PuzzleUserSerializer> = vec![];
    for puzzle_user in puzzle_users {
        let u: &User = users.get(&puzzle_user.sk).unwrap();
        puzzle_user_serializers.push(PuzzleUserSerializer::from((u, puzzle_user)));
    }

    return puzzle_user_serializers;
}

impl From<(&User, &PuzzleUser)> for PuzzleUserSerializer {
    fn from((user, puzzle_user): (&User, &PuzzleUser)) -> Self {
        PuzzleUserSerializer { 
            user: user.into(), 
            name: puzzle_user.name.clone(), 
            missing_pieces: puzzle_user.missing_pieces,
            puzzlers: puzzle_user.puzzlers,
            completion_time: puzzle_user.completion_time,
            media: puzzle_user.media.clone(),
            lat: puzzle_user.lat, 
            lng: puzzle_user.lng, 
            inserted: Some(puzzle_user.inserted.clone()),
            updated: Some(puzzle_user.updated.clone()),
        }

    }
}

impl From<&User> for UserSerializer {
    fn from(item: &User) -> Self {
        UserSerializer{
            user_id: Some(from_database_hash(&item.pk)),
            email: item.email.clone(),
            name: item.name.clone(),
            owned: item.owned.clone(),
            inserted: Some(item.inserted.clone()),
            updated: Some(item.updated.clone()),
        }
    }
}

impl From<&PuzzleUserSerializer> for User {
    fn from(item: &PuzzleUserSerializer) -> Self {
        let user_id = item.user.user_id.clone().unwrap();
        User{
            pk: to_database_hash(USER_PREFIX, &user_id),
            sk: to_database_hash(USER_PREFIX, &user_id),
            email: item.user.email.clone(),
            name: item.user.name.clone(),
            owned: item.user.owned.clone(),
            inserted: item.inserted.clone().unwrap(),
            updated: item.updated.clone().unwrap(),
        }
    }

}

impl From<&PuzzleSerializer> for Puzzle {
    fn from(item: &PuzzleSerializer) -> Self {
        let puzzle_id = &item.puzzle_id.clone().unwrap();
        Puzzle {
            pk: to_database_hash(PUZZLE_PREFIX, puzzle_id),
            sk: to_database_hash(PUZZLE_PREFIX, puzzle_id),
            title: item.title.clone(),
            media: item.media.clone(),
            num_pieces: item.num_pieces.clone(),
            inserted: item.inserted.clone().unwrap(),
            updated: item.updated.clone().unwrap(),
        }
    }
}

impl From<&Puzzle> for PuzzleSerializer {
    fn from(item: &Puzzle) -> Self {
        PuzzleSerializer{
            puzzle_id: Some(from_database_hash(&item.pk)),
            title: item.title.clone(),
            media: item.media.clone(),
            num_pieces: item.num_pieces.clone(),
            inserted: Some(item.inserted.clone()),
            updated: Some(item.updated.clone()),
            stamps: vec![] 

        }
    }
}
