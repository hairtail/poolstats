use std::{cmp, ops::Range, path::PathBuf, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{get, post},
    BoxError, Router,
};
use clap::Parser;
use log::info;
use poolstats::{
    poolstats::{get_nodes_info, overview_handler, Key},
    rpc::RpcHandler,
    DBHandler, Shared,
};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tokio::{net::TcpListener, time::sleep};
use tower::{timeout::TimeoutLayer, ServiceBuilder};
use tower_http::cors::{Any, CorsLayer};

fn get_default_db_path() -> PathBuf {
    std::env::current_dir().unwrap()
}

fn get_range(input: Range<i64>, batch: i64) -> Vec<Range<i64>> {
    let end = input.end;
    let mut result = vec![];
    for group in input.step_by(batch as usize) {
        let start = group;
        let end = cmp::min(start + batch - 1, end);
        result.push(start..end)
    }
    result
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
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");
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

    let fetch_resource = shared.clone();

    tokio::spawn(async move {
        loop {
            let epoch_info = fetch_resource
                .rpc_handler
                .get_epoch()
                .unwrap()
                .epochnum
                .number;
            let round_id = (epoch_info - 1).to_string();
            let mut next_round = None;
            let mut next_epoch = None;
            let current_layer = fetch_resource
                .rpc_handler
                .get_layer()
                .unwrap()
                .layernum
                .number;
            if current_layer >= epoch_info * 4032 + 2880 {
                next_round = Some(epoch_info.to_string());
                next_epoch = Some(epoch_info);
            }
            let count = fetch_resource
                .db_handler
                .count_initialzed()
                .await
                .unwrap_or(0);
            let limit = 50;
            for group in get_range(0..count, limit) {
                if let Ok(keys) = fetch_resource
                    .db_handler
                    .get_init_keys(limit, group.start)
                    .await
                {
                    info!("init posted keys {:?}", keys);
                    for Key { id, num_units } in keys {
                        match fetch_resource
                            .db_handler
                            .get_chain_registerations_by_id(id.clone(), round_id.clone())
                            .await
                        {
                            Ok(registerations) => {
                                if registerations.is_empty() {
                                    continue;
                                }
                                let _ = fetch_resource
                                    .db_handler
                                    .save_poet(id.clone(), num_units, registerations[0].clone())
                                    .await;
                            }
                            Err(_) => {}
                        }
                        match fetch_resource
                            .db_handler
                            .get_chain_atxs_by_id(id.clone(), epoch_info - 1)
                            .await
                        {
                            Ok(atx) => {
                                let _ = fetch_resource
                                    .db_handler
                                    .save_atx(id.clone(), num_units, atx)
                                    .await;
                            }
                            Err(e) => {
                                log::error!("{:?}", e)
                            }
                        }

                        if let Some(round_id) = next_round.clone() {
                            match fetch_resource
                                .db_handler
                                .get_chain_registerations_by_id(id.clone(), round_id.clone())
                                .await
                            {
                                Ok(registerations) => {
                                    if registerations.is_empty() {
                                        continue;
                                    }
                                    let _ = fetch_resource
                                        .db_handler
                                        .save_poet(id.clone(), num_units, registerations[0].clone())
                                        .await;
                                }
                                Err(_) => {}
                            }
                        }

                        if let Some(epoch) = next_epoch.clone() {
                            match fetch_resource
                                .db_handler
                                .get_chain_atxs_by_id(id.clone(), epoch)
                                .await
                            {
                                Ok(atx) => {
                                    let _ = fetch_resource
                                        .db_handler
                                        .save_atx(id.clone(), num_units, atx)
                                        .await;
                                }
                                Err(e) => {
                                    log::error!("{:?}", e)
                                }
                            }
                        }
                    }
                }
            }
            sleep(Duration::from_secs(30 * 60)).await;
        }
    });

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
