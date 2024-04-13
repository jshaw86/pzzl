use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_postgres::Client;
use futures_util::{pin_mut, TryStreamExt}; 

mod queries;

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    id: Option<i32>,
    email: String,
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Puzzle {
    id: String,
    name: String,
    media: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PuzzleUserSerializer {
    puzzle_id: Option<String>,
    name: String,
    media: String,
    users: Vec<User>,
}

#[derive(Clone)]
pub struct PzzlService {
   pub pool: Arc<Client>
}

impl PzzlService {
    pub async fn get_database_users(&self, puzzle_id: &str) -> Result<Vec<User>, tokio_postgres::Error> {
        let mut users = vec![];
        let database_users_result = self.pool
            .query_raw("SELECT u.id, u.email FROM users_puzzles AS up JOIN users AS u ON u.id = up.user_id WHERE puzzle_id = $1", &[puzzle_id])
            .await;

        return match database_users_result {
            Ok(database_users) => {
                pin_mut!(database_users);
                while let Some(row) = database_users.try_next().await.expect("failed row") {
                    users.push(User {
                        id: Some(row.get(0)),
                        email: row.get(1),
                    });
                }
                return Ok(users);
            },
            Err(err) => Err(err)

        }
    }

    pub async fn insert_users_puzzle(&self, name: &str, media: &str, email: &str) -> Result<PuzzleUserSerializer, tokio_postgres::Error> {
        // Insert the user into the database
        let row_result = self.pool
            .query_one(
                queries::INSERT_USERS_PUZZLE_STATEMENT,
                &[&name, &media, &email],
                )
            .await;

        return match row_result {
            Ok(row) =>  Ok(PuzzleUserSerializer {
                puzzle_id: row.get(1),
                name: row.get(2),
                media: row.get(3),
                users: vec![],
            }),
            Err(err) => Err(err) 
        }

    }

    pub async fn update_users_puzzle(&self, name: &str, media: &str, email: &str, puzzle_id: &str) -> Result<PuzzleUserSerializer, tokio_postgres::Error> {
        // Insert the user into the database
        let row_result = self.pool
            .query_one(
                queries::UPDATE_USERS_PUZZLE_STATEMENT,
                &[&name, &media, &email, &puzzle_id],
                )
            .await;

        return match row_result {
            Ok(row) =>  Ok(PuzzleUserSerializer {
                puzzle_id: row.get(1),
                name: row.get(2),
                media: row.get(3),
                users: vec![],
            }),
            Err(err) => Err(err) 
        }

    }

    // Function to create a new user
    pub async fn upsert_puzzle(&self, puzzle: PuzzleUserSerializer) -> Result<PuzzleUserSerializer, tokio_postgres::Error> {
        let mut new_puzzle: PuzzleUserSerializer = puzzle.clone();
        for user in &puzzle.users {
            let row_result = match &puzzle.puzzle_id {
                Some(puzzle_id) => self.update_users_puzzle(&puzzle.name, &puzzle.media, &user.email, puzzle_id.as_str()).await,
                None => self.insert_users_puzzle(&puzzle.name, &puzzle.media, &user.email).await

            };

            match row_result {
                Ok(puzzle_user) => puzzle_user,
                Err(err) => {
                    return Err(err) 
                }
            };

        }

        // update the cloned passed puzzle's users 
        new_puzzle.users = match self.get_database_users(&puzzle.puzzle_id.as_ref().unwrap()).await {
            Ok(users) => users,
            Err(err) => {
                return Err(err)
            }

        };

        return Ok(new_puzzle);

    }

    // Function to fetch a user's profile
    pub async fn get_puzzle(&self, id: String) -> Result<PuzzleUserSerializer, tokio_postgres::Error> {
        // fetch the puzzle metadata 
        let row = self.pool
            .query_one("SELECT id, name, media FROM puzzles WHERE id = $1", &[&id])
            .await
            .expect("Failed to fetch user");


        let puzzle_id: &String = &row.get(0);

        return match self.get_database_users(puzzle_id).await {
            Ok(users) => Ok(PuzzleUserSerializer {
                puzzle_id: Some(puzzle_id.to_string()),
                name: row.get(1),
                media: row.get(2),
                users,
            }),
            Err(err) => Err(err) 
        }; 
    }

}

