use std::{path::PathBuf, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{get, post},
    BoxError, Router,
};
use clap::Parser;
use log::info;
use poolstats::{
    poolstats::{get_nodes_info, overview_handler},
    rpc::RpcHandler,
    DBHandler, Shared,
};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tokio::net::TcpListener;
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::cors::{Any, CorsLayer};

fn get_default_db_path() -> PathBuf {
    std::env::current_dir().unwrap()
}

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
    /// datadir for cache db
    #[arg(short, long, default_value=get_default_db_path().into_os_string())]
    datadir: PathBuf,
    /// rpc node to query from
    #[arg(short, long)]
    node: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "debug");
    env_logger::init_from_env(env);
    let args = Cli::parse();
    info!("{:?}", args);
    let datadir = args.datadir;
    let db_url = datadir.join("poolstats.sql");
    let db_url = db_url.to_str().unwrap();
    if !Sqlite::database_exists(db_url).await.unwrap() {
        Sqlite::create_database(db_url).await.unwrap();
    }
    let poolstats = SqlitePool::connect(db_url).await.unwrap();
    sqlx::migrate!().run(&poolstats).await.unwrap();
    let db = SqlitePool::connect_lazy(&args.db)?;
    let local = SqlitePool::connect_lazy(&args.local)?;

    let db_handler = DBHandler::new(db, local, poolstats);
    let rpc_handler = RpcHandler::new(args.node);

    let shared = Shared::new(db_handler, rpc_handler);

    let router = Router::new()
        .route("/overview", get(overview_handler))
        .route("/nodes_info", post(get_nodes_info))
        .with_state(shared)
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
