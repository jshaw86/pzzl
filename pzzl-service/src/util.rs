use anyhow::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use aws_sdk_dynamodb::operation::execute_statement::ExecuteStatementOutput;
use serde_dynamo::from_item;
use crate::types::{User, Puzzle, PuzzleStamp};

pub fn parse_users(result: &ExecuteStatementOutput) -> Result<Vec<Arc<User>>> {
    let mut users: Vec<Arc<User>> = vec![];
    if let Some(items) = &result.items {
        for item in items {
            let user: User = from_item(item.clone())?;

            users.push(Arc::new(user));

        }

    }
    Ok(users)
}

pub fn parse_puzzle_stamps_users(result: &ExecuteStatementOutput) -> Result<(Option<Arc<Puzzle>>, Vec<Arc<PuzzleStamp>>, Vec<Arc<User>>)> {
    let mut puzzle_stamps: Vec<Arc<PuzzleStamp>> = vec![];
    let mut puzzle_users: Vec<Arc<User>> = vec![];
    let mut puzzle: Option<Arc<Puzzle>> = None;
    if let Some(items) = &result.items {
        for item in items {
            let puzzle_stamp_result: Result<PuzzleStamp, serde_dynamo::Error> = from_item(item.clone());
            if let Ok(stamp) = puzzle_stamp_result {
                puzzle_stamps.push(Arc::new(stamp));
                continue;
            }

            let user_result: Result<User, serde_dynamo::Error> = from_item(item.clone());
            if let Ok(user) = user_result {
                puzzle_users.push(Arc::new(user));
                continue
            }

            let puzzle_result: Result<Puzzle, serde_dynamo::Error> = from_item(item.clone());
            if let Ok(pzzl) =  puzzle_result {
                puzzle = Some(Arc::new(pzzl));
                continue
            }
             
            return Err(Error::msg(format!("unable to parse item {:?}", item)));
        }

    }
    return Ok((puzzle, puzzle_stamps, puzzle_users));

}

pub fn stamp_user_mapping(stamps: &Vec<Arc<PuzzleStamp>>, stamps_users: &Vec<Arc<User>>) -> HashMap<String, Vec<Arc<User>>> {
    let mut users_stamps: HashMap<std::string::String, Vec<Arc<User>>> = HashMap::new();
    for stamp in stamps {
        for user in stamps_users {
            match users_stamps.get_mut(&stamp.pk) {
                Some(stamps) => {
                    stamps.push(user.clone());
                },
                None => {
                    users_stamps.insert(stamp.sk.clone(), vec![user.clone()]);
                },
            };
        }
    }

    return users_stamps;

}

