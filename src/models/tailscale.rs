use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
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
    #[serde(rename = "LastSeen", deserialize_with = "parse_opt_datetime")]
    pub last_seen: Option<DateTime<Utc>>,
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

pub fn parse_opt_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(ref val) = s {
        // Treat the ancient default as "None"
        if val.starts_with("0001-01-01") {
            return Ok(None);
        }
        return DateTime::parse_from_rfc3339(val)
            .map(|dt| Some(dt.with_timezone(&Utc)))
            .map_err(serde::de::Error::custom);
    }
    Ok(None)
}
