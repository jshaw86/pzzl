use std::convert::From;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

const PUZZLE_PREFIX: &str = "PUZZLE";
//const PUZZLE_USER_PREFIX: &str = "PUZZLEUSER";
const USER_PREFIX: &str = "USER";
const DB_SEPARATOR: &str = "#";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSerializer {
    pub user_id: Option<String>,
    pub email: String,
    pub name: String,   
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleUserSerializer{
    pub user: UserSerializer,
    pub lat: f32,
    pub lng: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleSerializer {
    pub puzzle_id: Option<String>,
    pub name: String,
    pub media: String,
    pub users: Vec<PuzzleUserSerializer>,
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub pk: String,
    pub sk: String,
    pub email: String,
    pub name: String,   
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Puzzle {
    pub pk: String,
    pub sk: String,
    pub name: String,
    pub media: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleUser{
    pub pk: String,
    pub sk: String,
    pub lat: f32,
    pub lng: f32,
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
            lat: puzzle_user.lat,
            lng: puzzle_user.lng,
        }
    }
}

pub fn make_puzzle_user(user: &PuzzleUserSerializer, puzzle_id: &String) -> PuzzleUser {
    PuzzleUser {
        pk: to_database_hash(PUZZLE_PREFIX, &puzzle_id),
        sk: to_database_hash(USER_PREFIX, &user.user.user_id.clone().unwrap()),
        lat: user.lat,
        lng: user.lng,
    }
}

pub fn make_puzzle_user_serializers(users: &HashMap<String,User>, puzzle_users: &Vec<PuzzleUser>) -> Vec<PuzzleUserSerializer> {
    let mut puzzle_user_serializers: Vec<PuzzleUserSerializer> = vec![];
    for puzzle_user in puzzle_users {
        let u: &User = users.get(&puzzle_user.sk).unwrap();
        puzzle_user_serializers.push(make_puzzle_user_serializer(u, puzzle_user));
    }

    return puzzle_user_serializers;
}

fn make_puzzle_user_serializer(user: &User, puzzle_user: &PuzzleUser) -> PuzzleUserSerializer{
    PuzzleUserSerializer { user: user.into(), lat: puzzle_user.lat, lng: puzzle_user.lng }
}

impl From<(&User, &PuzzleUser)> for PuzzleUserSerializer {
    fn from((user, puzzle_user): (&User, &PuzzleUser)) -> Self {
        PuzzleUserSerializer { user: user.into(), lat: puzzle_user.lat, lng: puzzle_user.lng }

    }
}

impl From<&User> for UserSerializer {
    fn from(item: &User) -> Self {
        UserSerializer{
            user_id: Some(from_database_hash(&item.pk)),
            email: item.email.clone(),
            name: item.name.clone(),
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
        }
    }

}

impl From<&PuzzleSerializer> for Puzzle {
    fn from(item: &PuzzleSerializer) -> Self {
        let puzzle_id = &item.puzzle_id.clone().unwrap();
        Puzzle {
            pk: to_database_hash(PUZZLE_PREFIX, puzzle_id),
            sk: to_database_hash(PUZZLE_PREFIX, puzzle_id),
            name: item.name.clone(),
            media: item.media.clone(),
        }
    }
}

impl From<&Puzzle> for PuzzleSerializer {
    fn from(item: &Puzzle) -> Self {
        PuzzleSerializer{
            puzzle_id: Some(from_database_hash(&item.pk)),
            name: item.name.clone(),
            media: item.media.clone(),
            users: vec![] 

        }
    }
}
