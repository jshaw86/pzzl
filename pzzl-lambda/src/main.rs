use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client as DynamoClient;
use aws_sdk_s3::Client as S3Client;
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
use pzzl_service::{PzzlService, types::{PuzzleStampDeserializer, PuzzleDeserializer, MediaSerializer}};
use pzzl_service::types::PuzzleSerializer;
use pzzl_service::db::PzzlDatabase;
use tower_http::cors::{Any, CorsLayer};

const DOMAIN: &str = "https://puzzlepassport.com";
const DEFAULT_BUCKET: &str = "puzzle-passport-media";
const MAX_STAMPS_PER_REQ: usize = 4;

#[derive(Debug, Parser)]
pub struct Config {
    #[clap(long, env)]
    aws_endpoint: Option<String>,
    #[clap(long, env)]
    cors_origin: Option<String>,
    #[clap(long, env)]
    bucket_name: Option<String>,
}

#[derive(Clone)]
struct AppState {
   puzzle_service : PzzlService,
   bucket_name: String,
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

#[debug_handler]
async fn media_url(State(state): State<AppState>, Path(prefix): Path<String>) -> Result<Json<MediaSerializer>, AppError> {
    let result = state.puzzle_service.get_media_url(prefix, state.bucket_name).await;

    match result {
        Ok(s) => Ok(Json(s)),
        Err(e) => Err(AppError(e)),
    }
}

#[debug_handler]
async fn add_stamps(
    State(state): State<AppState>,
    Path(puzzle_id): Path<String>,
    Json(puzzle_users): Json<Vec<PuzzleStampDeserializer>>
) -> Result<Json<PuzzleSerializer>, AppError> {
    if puzzle_users.len() > MAX_STAMPS_PER_REQ {
        return Err(AppError(AnyError::msg("too many stamps")));
    }
    let puzzle_users_refs: Vec<&PuzzleStampDeserializer> = puzzle_users.iter().collect();
    let puzzle_result = state.puzzle_service.add_stamps(puzzle_id.as_str(), puzzle_users_refs).await;

    match puzzle_result {
        Ok(puzzle) => Ok(Json(puzzle)),
        Err(err) => Err(AppError(err)),
    }
}

// Function to create a new user
#[debug_handler]
async fn insert_puzzle(State(state): State<AppState>, Json(puzzle): Json<PuzzleDeserializer>) -> Result<Json<PuzzleSerializer>, AppError> {
    if puzzle.stamps.len() > MAX_STAMPS_PER_REQ {
        return Err(AppError(AnyError::msg("too many stamps")));
    }
    let puzzle_result = state.puzzle_service.insert_puzzle(&puzzle).await;

    return match puzzle_result {
        Ok(puzzle) => Ok(Json(puzzle)),
        Err(err) => Err(AppError(err)),
    };
    
}

// Function to fetch a user's profile
#[debug_handler]
async fn get_puzzle(State(state): State<AppState>, Path(puzzle_id): Path<String>) -> Result<Json<PuzzleSerializer>, AppError> {
    let puzzle_result = state.puzzle_service.get_puzzle(puzzle_id.as_str()).await;

    return match puzzle_result {
        Ok(puzzle) => Ok(Json(puzzle)),
        Err(err) => Err(AppError(err))
    }; 
}

async fn dynamo_client(aws_endpoint: &Option<String> ) -> DynamoClient {
    let config = match aws_endpoint {
        Some(url) =>  aws_config::defaults(BehaviorVersion::latest())
                .endpoint_url(url)
                .load().await,
        None => {
            aws_config::load_from_env().await
        }
    };
    

    return DynamoClient::new(&config);

}

async fn s3_client(aws_endpoint: &Option<String> ) -> S3Client {
    let config = match aws_endpoint {
        Some(url) =>  aws_config::defaults(BehaviorVersion::latest())
                .endpoint_url(url)
                .load().await,
        None => {
            aws_config::load_from_env().await
        }
    };
    

    return S3Client::new(&config);

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
            .with_max_level(tracing::Level::ERROR)
            .init();

    eprintln!("initialize dynamo client...");
    let dynamo_client = dynamo_client(&conf.aws_endpoint).await;
    let s3_client = s3_client(&conf.aws_endpoint).await;

    let resp = dynamo_client.list_tables().send().await?;

    let bucket_name = match conf.bucket_name {
        Some(bucket_name) => Some(bucket_name),
        None => Some(DEFAULT_BUCKET.to_string()),
    };

    eprintln!("Found {} tables", resp.table_names().len());

    let state = AppState{
        puzzle_service: PzzlService { 
            database: PzzlDatabase { client: Arc::new(dynamo_client) }, 
            s3_client: Arc::new(s3_client),

        },
        bucket_name: bucket_name.unwrap(),
    };

    // Create the Axum router
    let app = Router::new()
        .route("/health", get(|| async { "Hello, World!" }))
        .route("/media/:prefix", get(media_url))
        .route("/puzzles", put(insert_puzzle))
        .route("/puzzles/:puzzle_id", get(get_puzzle))
        .route("/puzzles/:puzzle_id/stamps", put(add_stamps))
        .layer(CorsLayer::new()
               .allow_methods(Any)
               .allow_origin(allowed_origin(conf.cors_origin))
               .allow_headers([CONTENT_TYPE]))
        .with_state(state);


    match conf.aws_endpoint {
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
