use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use anyhow::{Error, Result};
use aws_sdk_dynamodb::operation::execute_statement::ExecuteStatementOutput;
use aws_sdk_dynamodb::operation::batch_get_item::BatchGetItemOutput;
use serde_dynamo::from_item;
use uuid::Uuid;
use crate::types::PuzzleUserSerializer;
use crate::types::{User, Puzzle, PuzzleUser};




pub fn fill_user_id(puzzle_user: &PuzzleUserSerializer) -> PuzzleUserSerializer {
    match puzzle_user.user.user_id {
        Some(_) => puzzle_user.clone(),
        None => {
            let user_id = generate_user_id();
            let mut new_puzzle_user = puzzle_user.clone();
            new_puzzle_user.user.user_id = Some(user_id);
            new_puzzle_user
        }

    }

}

pub fn generate_user_id() -> String {
   Uuid::new_v4().to_string() 
}


pub fn generate_string(length: usize) -> String {
    let mut rng = rand::thread_rng();

    // Generate alphanumeric characters
    let alphanumeric_string: String = Alphanumeric.sample_string(&mut rng, length);

    // Replace some characters with special characters
    let final_string: Vec<char> = alphanumeric_string.chars().collect();

    final_string.into_iter().collect()
}

pub fn parse_users(result: &BatchGetItemOutput) -> Result<Vec<User>> {
    let responses = result.responses();

    if None == responses {
        return Err(Error::msg("no responses"));
    }
    let puzzle_users = responses.unwrap().get("puzzles_users");

    if None == puzzle_users {
        return Err(Error::msg("no puzzle_users"));
    }

    let mut users: Vec<User> = vec![];
    for item in puzzle_users.unwrap(){
        let user: User = from_item(item.clone())?;
            
        users.push(user);

    }

    Ok(users)

}

pub fn parse_puzzle_puzzle_users(result: &ExecuteStatementOutput) -> Result<(Puzzle, Vec<PuzzleUser>)> {
    let mut puzzle_users: Vec<PuzzleUser> = vec![];
    let mut puzzle: Option<Puzzle> = None;
    if let Some(items) = result.items.clone() {
        for item in items {
            let puzzle_user_result: Result<PuzzleUser, serde_dynamo::Error> = from_item(item.clone());
            let parsing_error: Result<(), serde_dynamo::Error> = match puzzle_user_result {
                Ok(user) => Ok(puzzle_users.push(user)),
                Err(_) => {
                    let puzzle_result: Result<Puzzle, serde_dynamo::Error> = from_item(item);
                    match puzzle_result {
                        Ok(returned_puzzle) => {
                            puzzle = Some(returned_puzzle);
                            Ok(())
                        },
                        Err(err) => Err(err),
                    }
                }
            };

            if let Err(serde_err) = parsing_error {
                return Err(Error::from(serde_err));
            }
        }

    }
    
    return Ok((puzzle.unwrap(), puzzle_users));

}
