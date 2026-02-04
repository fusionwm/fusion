use derive_more::From;
use serde::{Deserialize, Serialize};

pub const FUSION_CTL_SOCKET_DEFAULT: &str = "/tmp/fusion-ctl.sock";

#[derive(Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub status: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetPluginListRequest;

#[derive(Serialize, Deserialize)]
pub enum GetPluginListResponse {
    Ok(Vec<Plugin>),
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub struct RestartPluginRequest {
    pub plugin_id: String,
}
#[derive(Serialize, Deserialize)]
pub enum RestartPluginResponse {
    Ok,
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub struct PingRequest;
#[derive(Serialize, Deserialize)]
pub struct PingResponse;

#[derive(Serialize, Deserialize)]
pub struct ExitRequest;
#[derive(Serialize, Deserialize)]
pub struct ExitResponse;

#[derive(Serialize, Deserialize, From)]
pub enum CompositorRequest {
    Exit(ExitRequest),
    Ping(PingRequest),
    GetPluginList(GetPluginListRequest),
    RestartPlugin(RestartPluginRequest),
}
