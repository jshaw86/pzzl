use std::{convert::From, time::SystemTime};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::util;

const PUZZLE_PK_LENGTH: usize = 6;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaSerializer {
    pub uri: String,
    pub method: String,
    pub headers: Vec<(String, String)>, 
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
    pub num_puzzlers: u16,
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
    pub num_puzzlers: u16,
    pub missing_pieces: u16,
    pub completion_time: Option<u64>,
    pub urls: Vec<String>, // max six
    pub lat: f32,
    pub lng: f32,
    pub inserted: String,
    pub updated: String,
}

impl<'request> From<&'request PuzzleDeserializer> for PuzzleSerializer 
{
    fn from(item: &'request PuzzleDeserializer) -> Self {
        PuzzleSerializer {
            puzzle_id: util::generate_string(PUZZLE_PK_LENGTH).into(),
            title: item.title.to_string(),
            name: item.name.to_string(),
            url: item.url.to_string(),
            num_pieces: item.num_pieces,
            completion_time: item.completion_time,
            users: user_deserializer_to_serializer(&item.users),
            lat: item.lat,
            lng: item.lng,
            stamps: puzzle_stamp_deserializer_to_serializer(&item.stamps),
            inserted: util::rfc3339(&SystemTime::now()).into(),
            updated: util::rfc3339(&SystemTime::now()).into(),

        }
    }
}

impl<'request> From<&'request PuzzleStampDeserializer> for PuzzleStampSerializer 
{
    fn from(item: &'request PuzzleStampDeserializer) -> Self {
        PuzzleStampSerializer {
            stamp_id: Uuid::new_v4().to_string().into(),
            users: user_deserializer_to_serializer(&item.users),
            name: item.name.clone().map(String::from), 
            num_puzzlers: item.num_puzzlers,
            missing_pieces: item.missing_pieces,
            completion_time: item.completion_time,
            urls: item.urls.iter().map(|url| url.to_string()).collect(),
            lat: item.lat,
            lng: item.lng,
            inserted: util::rfc3339(&SystemTime::now()).into(),
            updated: util::rfc3339(&SystemTime::now()).into(),
        }
    }
}



impl<'request> From<&'request UserDeserializer> for UserSerializer 
{
    fn from(item: &'request UserDeserializer) -> Self {
        UserSerializer {
            user_id: Uuid::new_v4().to_string().into(),
            email: item.email.to_string(),
            name: item.name.to_string(), 
            inserted: util::rfc3339(&SystemTime::now()).into(),
            updated: util::rfc3339(&SystemTime::now()).into(),

        }
    }
}

fn user_deserializer_to_serializer<'request>(items: &'request Vec<UserDeserializer>) -> Vec<UserSerializer>
{
    items.into_iter().map(|item| item.into()).collect()
}

fn puzzle_stamp_deserializer_to_serializer<'request>(items: &'request Vec<PuzzleStampDeserializer>) -> Vec<PuzzleStampSerializer>
{
    items.into_iter().map(|item| item.into()).collect()

}
