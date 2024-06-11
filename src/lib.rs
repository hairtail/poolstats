use std::sync::Arc;

use axum::{
    extract::{self, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, Pool, Sqlite};

pub struct DBHandler {
    pub db: Pool<Sqlite>,
    pub local: Pool<Sqlite>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
pub struct Key {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
pub struct Registeration {
    pub address: String,
    pub round_id: String,
}

impl DBHandler {
    pub fn new(db: Pool<Sqlite>, local: Pool<Sqlite>) -> Arc<Self> {
        Arc::new(Self { db, local })
    }

    pub async fn get_all_initial_post_keys(&self) -> Result<Vec<Key>, sqlx::Error> {
        let result = sqlx::query_as("SELECT id FROM initial_post")
            .fetch_all(&self.local)
            .await?;
        Ok(result)
    }

    pub async fn get_registerations_by_id(
        &self,
        node_id: String,
    ) -> Result<Vec<Registeration>, sqlx::Error> {
        let result =
            sqlx::query_as("SELECT address, round_id FROM poet_registration where id = $1")
                .bind(node_id)
                .fetch_all(&self.local)
                .await?;
        Ok(result)
    }
}

pub async fn overview_handler(State(db_handler): State<Arc<DBHandler>>) -> impl IntoResponse {
    match db_handler.get_all_initial_post_keys().await {
        Ok(online_keys) => Json(json!({"count": online_keys.len(), "keys": online_keys})),
        Err(_) => Json(json!({"count": 0, "keys": []})),
    }
}

pub async fn registerations_handler(
    State(db_handler): State<Arc<DBHandler>>,
    extract::Json(node_id): extract::Json<String>,
) -> impl IntoResponse {
    match db_handler.get_registerations_by_id(node_id).await {
        Ok(registerations) => {
            Json(json!({"count": registerations.len(), "registerations": registerations}))
        }
        Err(_) => Json(json!({"count": 0, "registerations": []})),
    }
}
