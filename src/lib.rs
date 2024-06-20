use std::sync::Arc;

pub mod chain;
pub mod poolstats;
pub mod rpc;

use rpc::RpcHandler;
use sqlx::{Pool, Sqlite};

pub struct DBHandler {
    pub chain: Pool<Sqlite>,
    pub local: Pool<Sqlite>,
    pub poolstats: Pool<Sqlite>,
}

impl DBHandler {
    pub fn new(chain: Pool<Sqlite>, local: Pool<Sqlite>, poolstats: Pool<Sqlite>) -> Self {
        Self {
            chain,
            local,
            poolstats,
        }
    }
}

pub struct Shared {
    pub db_handler: DBHandler,
    pub rpc_handler: RpcHandler,
}

impl Shared {
    pub fn new(db_handler: DBHandler, rpc_handler: RpcHandler) -> Arc<Self> {
        Arc::new(Self {
            db_handler,
            rpc_handler,
        })
    }
}
