use axum::{
    debug_handler,
    routing::{get, post},
    extract::{Json, State, Path},
    http::StatusCode,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};
use futures_util::{pin_mut, TryStreamExt}; 

mod queries;

#[derive(Clone)]
struct AppState {
    pool: Arc<Client>,
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: Option<i32>,
    email: String,
}

// User model
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Puzzle {
    id: String,
    name: String,
    media: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PuzzleUserSerializer {
    puzzle_id: Option<String>,
    name: String,
    media: String,
    users: Vec<User>,
}

async fn get_database_users(pool: Arc<Client>, puzzle_id: &str) -> Result<Vec<User>, tokio_postgres::Error> {
 let mut users = vec![];
 let database_users_result = pool
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

async fn insert_users_puzzle(pool: Arc<Client>, name: &str, media: &str, email: &str) -> Result<PuzzleUserSerializer, tokio_postgres::Error> {
        // Insert the user into the database
        let row_result = pool
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

async fn update_users_puzzle(pool: Arc<Client>, name: &str, media: &str, email: &str, puzzle_id: &str) -> Result<PuzzleUserSerializer, tokio_postgres::Error> {
        // Insert the user into the database
        let row_result = pool
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
#[debug_handler]
async fn upsert_puzzle(State(state): State<AppState>, puzzle: Json<PuzzleUserSerializer>) -> Result<Json<PuzzleUserSerializer>, StatusCode> {
    let mut new_puzzle: Json<PuzzleUserSerializer> = puzzle.clone();
    for user in &puzzle.users {
        let row_result = match &puzzle.puzzle_id {
            Some(puzzle_id) => update_users_puzzle(state.pool.clone(), &puzzle.name, &puzzle.media, &user.email, puzzle_id.as_str()).await,
            None => insert_users_puzzle(state.pool.clone(), &puzzle.name, &puzzle.media, &user.email).await

        };

        match row_result {
                Ok(puzzle_user) => puzzle_user,
                Err(err) => {
                    println!("{}", err);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

    }

    // update the cloned passed puzzle's users 
    new_puzzle.users = match get_database_users(state.pool, &puzzle.puzzle_id.as_ref().unwrap()).await {
        Ok(users) => users,
        Err(err) => {
            println!("{}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        }

    };

    return Ok(new_puzzle);
    
}

// Function to fetch a user's profile
#[debug_handler]
async fn get_puzzle(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<PuzzleUserSerializer>, StatusCode> {
    // fetch the puzzle metadata 
    let row = state.pool
        .query_one("SELECT id, name, media FROM puzzles WHERE id = $1", &[&id])
        .await
        .expect("Failed to fetch user");


    let puzzle_id: &String = &row.get(0);

    return match get_database_users(state.pool, puzzle_id).await {
        Ok(users) => Ok(Json(PuzzleUserSerializer {
            puzzle_id: Some(puzzle_id.to_string()),
            name: row.get(1),
            media: row.get(2),
            users,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }; 
}

#[tokio::main]
async fn main() {
    let db_connection_str = "postgresql://postgres:mysecretpassword@localhost:5432/?connect_timeout=10";

    // Connect to the PostgreSQL database
    let (client, connection) = tokio_postgres::connect(db_connection_str, NoTls)
        .await
        .expect("Failed to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            println!("Failed to connect to database: {}", e);
        }
    });

    let state = AppState{
        pool: Arc::new(client)
    };

    // Create the Axum router
    let app = Router::new()
        .route("/puzzles", post(upsert_puzzle))
        .route("/puzzles/:id", get(get_puzzle))
        .with_state(state);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8089").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

