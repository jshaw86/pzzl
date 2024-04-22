use lambda_http::{run, tracing, Error};
use axum::{
    debug_handler,
    routing::{get, post},
    extract::{Json, State, Path},
    http::StatusCode,
    Router,
};
use pzzl_service::{PzzlService, UserPuzzleSerializer};
use std::sync::Arc;
use tokio_postgres::NoTls;
use std::env::set_var;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Config {
    #[clap(long, env)]
    database_url: String,
    #[clap(long, env)]
    database_user: String,
    #[clap(long, env)]
    database_password: String,
    #[clap(long, env)]
    database_timeout: u8,
}

#[derive(Clone)]
struct AppState {
   puzzle_service : PzzlService,
}

// Function to create a new user
#[debug_handler]
async fn upsert_puzzle(State(state): State<AppState>, Json(puzzle): Json<UserPuzzleSerializer>) -> Result<Json<UserPuzzleSerializer>, StatusCode> {
    let row_result = state.puzzle_service.upsert_puzzle(puzzle).await;

    return match row_result {
        Ok(puzzle_user) => Ok(Json(puzzle_user)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
}

// Function to fetch a user's profile
#[debug_handler]
async fn get_puzzle(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<UserPuzzleSerializer>, StatusCode> {
    let row_result = state.puzzle_service.get_puzzle(id).await;

    return match row_result {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }; 
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");
    let conf = Config::parse();

    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();
    let db_connection_str = format!("postgresql://{user}:{password}@{url}/?connect_timeout={timeout}", 
                                    user=conf.database_user, password=conf.database_password, url=conf.database_url, timeout=conf.database_timeout);

    // Connect to the PostgreSQL database
    let (client, connection) = tokio_postgres::connect(&db_connection_str, NoTls)
        .await
        .expect("Failed to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            println!("Failed to connect to database: {}", e);
        }
    });

    let state = AppState{
        puzzle_service: PzzlService { pool: Arc::new(client) }
    };

    // Create the Axum router
    let app = Router::new()
        .route("/puzzles", post(upsert_puzzle))
        .route("/puzzles/:id", get(get_puzzle))
        .with_state(state);

    // Run the server
    let _ = run(app).await;
    Ok(())

}

