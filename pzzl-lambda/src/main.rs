use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use anyhow::Error as AnyError;
use lambda_http;
use lambda_http::{tracing, Error as LambdaError};
use axum::{
    debug_handler,
    routing::{get, put},
    extract::{Json, Path, FromRequest, State},
    Router,
    response::{IntoResponse, Response},
    http::{HeaderValue, StatusCode},
    http::header::CONTENT_TYPE,
};
use std::sync::Arc;
//use std::env::set_var;
use serde::Serialize;
use clap::Parser;
use pzzl_service::{PzzlService, types::PuzzleUserSerializer};
use pzzl_service::types::PuzzleSerializer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

const DOMAIN: &str = "https://puzzlepassport.com";
const MAX_STAMPS_PER_REQ: usize = 4;

#[derive(Debug, Parser)]
pub struct Config {
    #[clap(long, env)]
    dynamo_endpoint: Option<String>,
    #[clap(long, env)]
    cors_origin: Option<String>,
}

#[derive(Clone)]
struct AppState {
   puzzle_service : PzzlService,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
    status_code: u16,
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
struct AppJson<T>(T);

struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse{
                status_code: StatusCode::BAD_REQUEST.into(),
                message: format!("{}", self.0)
            }),
        ).into_response()
    }
}
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[debug_handler]
async fn add_stamps(State(state): State<AppState>, Path(puzzle_id): Path<String>, Json(puzzle_users): Json<Vec<PuzzleUserSerializer>>) -> Result<Json<PuzzleSerializer>, AppError> {
    if puzzle_users.len() > MAX_STAMPS_PER_REQ {
        return Err(AppError(AnyError::msg("too many stamps")));
    }
    let puzzle_result = state.puzzle_service.add_stamps(puzzle_id, puzzle_users).await;

    return match puzzle_result {
        Ok(puzzle) => Ok(Json(puzzle)),
        Err(err) => Err(AppError(err)),
    };

}

// Function to create a new user
#[debug_handler]
async fn insert_puzzle(State(state): State<AppState>, Json(puzzle): Json<PuzzleSerializer>) -> Result<Json<PuzzleSerializer>, AppError> {
    if puzzle.stamps.len() > MAX_STAMPS_PER_REQ {
        return Err(AppError(AnyError::msg("too many stamps")));
    }
    let puzzle_result = state.puzzle_service.insert_puzzle(puzzle).await;

    return match puzzle_result {
        Ok(puzzle) => Ok(Json(puzzle)),
        Err(err) => Err(AppError(err)),
    };
    
}

// Function to fetch a user's profile
#[debug_handler]
async fn get_puzzle(State(state): State<AppState>, Path(puzzle_id): Path<String>) -> Result<Json<PuzzleSerializer>, AppError> {
    let puzzle_result = state.puzzle_service.get_puzzle(puzzle_id).await;

    return match puzzle_result {
        Ok(puzzle) => Ok(Json(puzzle)),
        Err(err) => Err(AppError(err))
    }; 
}

async fn dynamo_client(dynamo_endpoint: &Option<String> ) -> Client {
    let config = match dynamo_endpoint {
        Some(url) =>  aws_config::defaults(BehaviorVersion::latest())
                .endpoint_url(url)
                .load().await,
        None => {
            eprintln!("loading dynamo from env...");
            aws_config::load_from_env().await
        }
    };
    

    return Client::new(&config);

}

fn allowed_origin(cors_origin: Option<String>) -> HeaderValue {
    let allowed_origin: String = match cors_origin {
        Some(origin) => origin,
        None => DOMAIN.to_string(),
        
    };
   allowed_origin.parse::<HeaderValue>().unwrap()
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    eprintln!("starting up...");
    //set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");
    // required to enable CloudWatch error logging by the runtime

    let conf = Config::parse();

    tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

    eprintln!("initialize dynamo client...");
    let client = dynamo_client(&conf.dynamo_endpoint).await;

    let resp = client.list_tables().send().await?;

    eprintln!("Found {} tables", resp.table_names().len());

    let state = AppState{
        puzzle_service: PzzlService { pool: Arc::new(client) }
    };

    // Create the Axum router
    let app = Router::new()
        .route("/health", get(|| async { "Hello, World!" }))
        .route("/puzzles", put(insert_puzzle))
        .route("/puzzles/:puzzle_id", get(get_puzzle))
        .route("/puzzles/:puzzle_id/stamps", put(add_stamps))
        .layer(CorsLayer::new()
               .allow_methods(Any)
               .allow_origin(allowed_origin(conf.cors_origin))
               .allow_headers([CONTENT_TYPE]))
        .layer(TraceLayer::new_for_http())
        .with_state(state);


    match conf.dynamo_endpoint {
        Some(_) => {
            let listener = tokio::net::TcpListener::bind("0.0.0.0:8089").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        None => {
            eprintln!("running dynamo app...");
            let resp = lambda_http::run(app).await;
            eprintln!("dynamo app resp {:?}", resp);
        }
    }
    Ok(())

}

