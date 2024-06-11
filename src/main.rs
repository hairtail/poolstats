use std::time::Duration;

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{get, post},
    BoxError, Router,
};
use clap::Parser;
use log::info;
use poolstats::{overview_handler, registerations_handler, DBHandler};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::cors::{Any, CorsLayer};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// sqlite db path
    #[arg(short, long)]
    db: String,
    /// sqlite local db path
    #[arg(short, long)]
    local: String,
    /// ip:port to listen for api request
    #[arg(short, long)]
    listen: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "debug");
    env_logger::init_from_env(env);
    let args = Cli::parse();
    info!("{:?}", args);
    let db = SqlitePool::connect_lazy(&args.db)?;
    let local = SqlitePool::connect_lazy(&args.local)?;

    let db_handler = DBHandler::new(db, local);

    let router = Router::new()
        .route("/overview", get(overview_handler))
        .route("/registerations", post(registerations_handler))
        .with_state(db_handler)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    StatusCode::REQUEST_TIMEOUT
                }))
                .layer(TimeoutLayer::new(Duration::from_secs(30))),
        )
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any),
        );
    let listener = TcpListener::bind(&args.listen).await?;
    info!("api server starts listening at {}", args.listen);
    axum::serve(listener, router).await?;
    Ok(())
}
