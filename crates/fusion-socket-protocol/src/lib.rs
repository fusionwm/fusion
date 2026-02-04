use serde::{Deserialize, Serialize};

pub const FUSION_CTL_SOCKET_VAR: &str = "FUSION_CTL_SOCKET";
pub const FUSION_CTL_SOCKET_DEFAULT: &str = "/tmp/fusion-ctl.sock";

#[derive(Serialize, Deserialize)]
pub enum CompositorRequest {
    GetPlugins,
    Restart { plugin_id: String },
}

#[derive(Serialize, Deserialize)]
pub enum CompositorResponse {
    Plugins(Vec<Plugin>),
    Ok,
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub status: String,
    pub version: String,
}
