use crate::models::service::ServiceView;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct TailscaleStatus {
    #[serde(rename = "Self")]
    pub self_device: Option<Device>,

    #[serde(rename = "Peer")]
    pub peers: Option<HashMap<String, Device>>,

    #[serde(rename = "User")]
    pub users: Option<HashMap<String, User>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Device {
    #[serde(rename = "ID")]
    pub id: Option<String>,
    #[serde(rename = "HostName")]
    pub host_name: Option<String>,
    #[serde(rename = "OS")]
    pub os: Option<String>,
    #[serde(rename = "TailscaleIPs")]
    pub ips: Option<Vec<String>>,
    #[serde(rename = "UserID")]
    pub user_id: Option<i64>,
    #[serde(rename = "Online")]
    pub online: Option<bool>,
    #[serde(rename = "PeerAPIURL")]
    pub peer_api_url: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "ID")]
    pub id: Option<i64>,
    #[serde(rename = "LoginName")]
    pub login_name: Option<String>,
    #[serde(rename = "DisplayName")]
    pub display_name: Option<String>,
}

#[derive(Clone)]
pub struct DeviceView {
    pub host_name: String,
    pub ip: String,
    pub os: String,
    pub online: bool,
    pub services: Vec<ServiceView>,
    pub internal_services: Vec<ServiceView>,
}
