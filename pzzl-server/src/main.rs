use axum::{
    debug_handler,
    routing::{get, post},
    extract::{Json, State, Path},
    http::StatusCode,
    Router,
};
use pzzl_service::{PzzlService, PuzzleUserSerializer};
use std::sync::Arc;
use tokio_postgres::NoTls;

#[derive(Clone)]
struct AppState {
   puzzle_service : PzzlService,
}

// Function to create a new user
#[debug_handler]
async fn upsert_puzzle(State(state): State<AppState>, Json(puzzle): Json<PuzzleUserSerializer>) -> Result<Json<PuzzleUserSerializer>, StatusCode> {
    let row_result = state.puzzle_service.upsert_puzzle(puzzle).await;

    return match row_result {
        Ok(puzzle_user) => Ok(Json(puzzle_user)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
}

// Function to fetch a user's profile
#[debug_handler]
async fn get_puzzle(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<PuzzleUserSerializer>, StatusCode> {
    let row_result = state.puzzle_service.get_puzzle(id).await;

    return match row_result {
        Ok(result) => Ok(Json(result)),
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
        puzzle_service: PzzlService { pool: Arc::new(client) }
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
