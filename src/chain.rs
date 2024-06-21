use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    poolstats::{AtxInfo, Key, Registeration},
    DBHandler,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
struct InnerKey {
    pub id: Vec<u8>,
    pub num_units: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
pub struct InnerAtxInfo {
    pub epoch: i64,
    pub atx_id: Vec<u8>,
    pub effective_num_units: i64,
    pub coinbase: Vec<u8>,
}

impl InnerAtxInfo {
    pub fn to_atx(self) -> AtxInfo {
        let InnerAtxInfo {
            epoch,
            atx_id,
            effective_num_units,
            coinbase,
        } = self;
        AtxInfo {
            atx_id: hex::encode(atx_id),
            effective_num_units,
            epoch,
            coinbase: hex::encode(coinbase),
        }
    }
}

impl DBHandler {
    pub async fn get_init_keys(&self, limit: i64, offset: i64) -> Result<Vec<Key>, sqlx::Error> {
        let result: Vec<InnerKey> =
            sqlx::query_as("SELECT id, num_units FROM initial_post LIMIT $1 OFFSET $2")
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.local)
                .await?;
        let result = result
            .into_iter()
            .map(|k| Key {
                id: hex::encode(k.id),
                num_units: k.num_units,
            })
            .collect();
        Ok(result)
    }

    pub async fn count_initialzed(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar("SELECT COUNT (*) FROM initial_post")
            .fetch_one(&self.local)
            .await?;
        Ok(result)
    }

    pub async fn inited_num_units(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar("SELECT SUM (num_units) FROM initial_post")
            .fetch_one(&self.local)
            .await?;
        Ok(result)
    }

    pub async fn get_chain_registerations_by_id(
        &self,
        id: String,
        round_id: String,
    ) -> Result<Vec<Registeration>, sqlx::Error> {
        let result = sqlx::query_as(
            "SELECT address, round_id, round_end FROM poet_registration WHERE id = $1 AND round_id = $2",
        )
        .bind(hex::decode(id).unwrap())
        .bind(round_id)
        .fetch_all(&self.local)
        .await?;
        Ok(result)
    }

    pub async fn get_chain_atxs_by_id(
        &self,
        id: String,
        epoch: i64,
    ) -> Result<AtxInfo, sqlx::Error> {
        let result = sqlx::query_as::<_, InnerAtxInfo>(
            "SELECT epoch, id AS atx_id, effective_num_units, coinbase FROM atxs WHERE pubkey = $1 AND epoch = $2",
        )
        .bind(hex::decode(id).unwrap())
        .bind(epoch)
        .fetch_one(&self.chain)
        .await?;
        Ok(result.to_atx())
    }
}
