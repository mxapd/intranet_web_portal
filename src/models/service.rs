use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, serde::Serialize)]
pub struct ServiceConfig {
    pub service: Vec<ServiceEntry>,
}

#[derive(Debug, Deserialize, Clone, serde::Serialize)]
pub struct ServiceEntry {
    pub host: String,
    pub name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddServiceForm {
    pub host: String,
    pub name: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoveServiceForm {
    pub index: usize,
}
