use std::sync::Arc;

use axum::{
    extract::{self, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

use crate::{DBHandler, Shared};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneralRequest {
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
pub struct Key {
    pub id: String,
    pub num_units: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
pub struct Registeration {
    pub address: String,
    pub round_id: String,
    pub round_end: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
pub struct AtxInfo {
    pub epoch: i64,
    pub atx_id: String,
    pub effective_num_units: i64,
    pub coinbase: String,
}

impl AtxInfo {
    pub fn empty(&self) -> bool {
        self.epoch == 0
    }
}

impl Default for AtxInfo {
    fn default() -> Self {
        Self {
            epoch: 0,
            atx_id: Default::default(),
            effective_num_units: 0,
            coinbase: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeInfo {
    pub id: String,
    pub num_units: i64,
    pub registerations: Vec<Registeration>,
    pub atx: AtxInfo,
}

impl NodeInfo {
    fn new(id: String, num_units: i64, registerations: Vec<Registeration>, atx: AtxInfo) -> Self {
        Self {
            id,
            num_units,
            registerations,
            atx,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    pub count: i64,
    pub num_units: i64,
}

impl Item {
    fn new(count: i64, num_units: i64) -> Self {
        Self { count, num_units }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralItem {
    pub current: Item,
    pub next: Item,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Overview {
    pub init_posted: Item,
    pub registerd: GeneralItem,
    pub actived: GeneralItem,
}

impl IntoResponse for Overview {
    fn into_response(self) -> axum::response::Response {
        Json(json!({"code": 200, "data": self})).into_response()
    }
}

impl DBHandler {
    pub async fn save_poet(
        &self,
        id: String,
        num_unit: i64,
        poet: Registeration,
    ) -> Result<(), sqlx::Error> {
        let _ = sqlx::query(
            "INSERT INTO poet_registration (id, round_id, num_unit) VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(poet.round_id)
        .bind(num_unit)
        .fetch_one(&self.poolstats)
        .await?;
        Ok(())
    }

    pub async fn save_atx(
        &self,
        id: String,
        num_unit: i64,
        atx: AtxInfo,
    ) -> Result<(), sqlx::Error> {
        let _ = sqlx::query(
            "INSERT INTO atxs (id, epoch, effective_num_units, coinbase, atx_id, num_unit) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(atx.epoch)
        .bind(atx.effective_num_units)
        .bind(atx.coinbase)
        .bind(atx.atx_id)
        .bind(num_unit)
        .fetch_one(&self.poolstats)
        .await?;
        Ok(())
    }
}

impl DBHandler {
    pub async fn count_activated(&self, epoch: i64) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar("SELECT COUNT (*) FROM atxs WHERE epoch = $1")
            .bind(epoch)
            .fetch_one(&self.poolstats)
            .await?;
        Ok(result)
    }

    pub async fn actived_num_units(&self, epoch: i64) -> Result<i64, sqlx::Error> {
        let result =
            sqlx::query_scalar("SELECT SUM (effective_num_units) FROM atxs WHERE epoch = $1")
                .bind(epoch)
                .fetch_one(&self.poolstats)
                .await?;
        Ok(result)
    }

    pub async fn count_registered(&self, round_id: String) -> Result<i64, sqlx::Error> {
        let result =
            sqlx::query_scalar("SELECT COUNT (*) FROM poet_registration WHERE round_id = $1")
                .bind(round_id)
                .fetch_one(&self.poolstats)
                .await?;
        Ok(result)
    }

    pub async fn registered_num_units(&self, round_id: String) -> Result<i64, sqlx::Error> {
        let result =
            sqlx::query_scalar("SELECT SUM (num_unit) FROM poet_registration WHERE round_id = $1")
                .bind(round_id)
                .fetch_one(&self.poolstats)
                .await?;
        Ok(result)
    }

    pub async fn get_atxs_by_id(&self, id: String, epoch: i64) -> Result<AtxInfo, sqlx::Error> {
        let result = sqlx::query_as(
            "SELECT epoch, atx_id, effective_num_units, coinbase FROM atxs WHERE id = $1 AND epoch = $2",
        )
        .bind(hex::decode(id).unwrap())
        .bind(epoch)
        .fetch_one(&self.poolstats)
        .await?;
        Ok(result)
    }
}

pub async fn overview_handler(State(shared): State<Arc<Shared>>) -> impl IntoResponse {
    let init_posted_count = shared.db_handler.count_initialzed().await.unwrap_or(0);
    let init_posted_num_units = shared.db_handler.inited_num_units().await.unwrap_or(0);

    let epoch_info = shared.rpc_handler.get_epoch().unwrap().epochnum.number;
    let round_id = (epoch_info - 1).to_string();

    let registed_count = shared
        .db_handler
        .count_registered(round_id.clone())
        .await
        .unwrap_or(0);
    let registed_num_units = shared
        .db_handler
        .registered_num_units(round_id.clone())
        .await
        .unwrap_or(0);

    let next_round_id = epoch_info.to_string();
    let next_registed_count = shared
        .db_handler
        .count_registered(next_round_id.clone())
        .await
        .unwrap_or(0);
    let next_registed_num_units = shared
        .db_handler
        .registered_num_units(next_round_id.clone())
        .await
        .unwrap_or(0);

    let actived_count = shared
        .db_handler
        .count_activated(epoch_info - 1)
        .await
        .unwrap_or(0);
    let actived_num_units = shared
        .db_handler
        .actived_num_units(epoch_info - 1)
        .await
        .unwrap_or(0);

    let next_actived_count = shared
        .db_handler
        .count_activated(epoch_info)
        .await
        .unwrap_or(0);
    let next_actived_num_units = shared
        .db_handler
        .actived_num_units(epoch_info)
        .await
        .unwrap_or(0);
    Overview {
        init_posted: Item::new(init_posted_count, init_posted_num_units),
        registerd: GeneralItem {
            current: Item::new(registed_count, registed_num_units),
            next: Item::new(next_registed_count, next_registed_num_units),
        },
        actived: GeneralItem {
            current: Item::new(actived_count, actived_num_units),
            next: Item::new(next_actived_count, next_actived_num_units),
        },
    }
}

pub async fn get_nodes_info(
    State(shared): State<Arc<Shared>>,
    extract::Json(req): extract::Json<GeneralRequest>,
) -> impl IntoResponse {
    let GeneralRequest { limit, offset } = req;
    let ids = shared
        .db_handler
        .get_init_keys(limit, offset)
        .await
        .unwrap_or(vec![]);
    let mut result = vec![];
    let epoch_info = shared.rpc_handler.get_epoch().unwrap().epochnum.number;
    let mut round_id = (epoch_info - 1).to_string();
    let current_layer = shared.rpc_handler.get_layer().unwrap().layernum.number;
    if current_layer >= epoch_info * 4032 + 2880 {
        round_id = epoch_info.to_string();
    }
    for Key { id, num_units } in ids {
        let registerations = shared
            .db_handler
            .get_chain_registerations_by_id(id.clone(), round_id.clone())
            .await
            .unwrap_or(vec![]);
        let atx = shared
            .db_handler
            .get_atxs_by_id(id.clone(), epoch_info - 1)
            .await
            .unwrap_or(AtxInfo::default());
        result.push(NodeInfo::new(id, num_units, registerations, atx));
    }
    Json(json!({"code": 200, "data": json!({"data": result})})).into_response()
}
