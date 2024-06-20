use std::{fmt::Debug, time::Duration};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder, Error, Response};

#[derive(Debug, Deserialize, Serialize)]
pub struct Number {
    pub number: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EpochInfo {
    pub epochnum: Number,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LayerInfo {
    pub layernum: Number,
}

#[derive(Debug, Clone)]
pub struct RpcHandler {
    pub endpoint: String,
    pub agent: Agent,
}

impl RpcHandler {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            agent: AgentBuilder::new()
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build(),
        }
    }

    pub fn get_epoch(&self) -> anyhow::Result<EpochInfo> {
        let path = format!("http://{}/v1/mesh/currentepoch", self.endpoint);
        let resp = self.agent.clone().post(&path).call();
        handle_response(resp)
    }

    pub fn get_layer(&self) -> anyhow::Result<LayerInfo> {
        let path = format!("http://{}/v1/mesh/currentlayer", self.endpoint);
        let resp = self.agent.clone().post(&path).call();
        handle_response(resp)
    }
}

pub fn handle_response<S: Debug + for<'a> Deserialize<'a>>(
    resp: Result<Response, Error>,
) -> anyhow::Result<S> {
    let res = match resp {
        Ok(response) => match response.into_json::<S>() {
            Ok(data) => Ok(data),
            Err(e) => Err(anyhow!(e.to_string())),
        },
        Err(e) => Err(anyhow!(e.to_string())),
    };
    res
}
