use askama::Template;
use axum::{Router, response::Html, routing::get};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(serve_index));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ===== ROUTE HANDLER =====
async fn serve_index() -> Html<String> {
    match (get_tailscale_status().await, load_services_config()) {
        (Ok(status), Ok(services_cfg)) => {
            let grouped = group_by_user(&status, &services_cfg);
            let quick_services = extract_all_services(&grouped, &services_cfg); // ✅ here
            let template = PortalTemplate {
                users: &grouped,
                quick_services: &quick_services,
            };
            Html(
                template
                    .render()
                    .unwrap_or_else(|_| "Template render error".to_string()),
            )
        }
        (Err(e), _) => {
            eprintln!("Error fetching Tailscale data: {e}");
            Html("Error fetching data".into())
        }
        (_, Err(e)) => {
            eprintln!("Error loading services config: {e}");
            Html("Error loading services config".into())
        }
    }
}

// ===== PROCESS LOGIC =====
async fn get_tailscale_status() -> anyhow::Result<TailscaleStatus> {
    let mut cmd = Command::new("tailscale");
    cmd.args(["status", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = cmd.spawn()?;

    let mut output = vec![];
    if let Some(mut out) = child.stdout.take() {
        out.read_to_end(&mut output).await?;
    }

    let status: TailscaleStatus = serde_json::from_slice(&output)?;
    Ok(status)
}

fn load_services_config() -> anyhow::Result<ServiceConfig> {
    let data = std::fs::read_to_string("config/services.toml")?;
    let config: ServiceConfig = toml::from_str(&data)?;
    Ok(config)
}

// ===== TEMPLATE =====
#[derive(Template)]
#[template(path = "index.html")]
struct PortalTemplate<'a> {
    users: &'a BTreeMap<String, Vec<DeviceView>>,
    quick_services: &'a Vec<ServiceView>,
}

// ===== GROUPING =====
fn group_by_user(
    status: &TailscaleStatus,
    svc_cfg: &ServiceConfig,
) -> BTreeMap<String, Vec<DeviceView>> {
    let mut map: BTreeMap<String, Vec<DeviceView>> = BTreeMap::new();
    let mut all_devices = Vec::new();

    if let Some(self_dev) = &status.self_device {
        all_devices.push(self_dev.clone());
    }
    if let Some(peers) = &status.peers {
        all_devices.extend(peers.values().cloned());
    }

    if let Some(users) = &status.users {
        for dev in all_devices {
            let uname = dev
                .user_id
                .and_then(|uid| users.get(&uid.to_string()))
                .and_then(|u| u.login_name.clone())
                .unwrap_or_else(|| "unknown".to_string());

            // normal services loaded from config
            let services = svc_cfg
                .service
                .iter()
                .filter(|s| s.host == dev.host_name.clone().unwrap_or_default())
                .map(|s| ServiceView {
                    name: s.name.clone(),
                    owner: uname.clone(),
                    url: s.url.clone(),
                })
                .collect::<Vec<_>>();

            // internal utilities (Peer API, debugging)
            let mut internal_services = Vec::new();

            if let Some(urls) = dev.peer_api_url.as_ref() {
                if let Some(u) = urls.iter().find(|x| x.starts_with("http://100.")) {
                    internal_services.push(ServiceView {
                        name: "Peer API".to_string(),
                        owner: uname.clone(),
                        url: u.clone(),
                    });
                }
            }

            let view = DeviceView {
                host_name: dev.host_name.clone().unwrap_or_else(|| "unnamed".into()),
                ip: dev
                    .ips
                    .as_ref()
                    .and_then(|v| v.iter().find(|ip| ip.starts_with("100.")))
                    .cloned()
                    .unwrap_or_else(|| "?".into()),
                os: dev.os.clone().unwrap_or_else(|| "unknown".into()),
                online: dev.online.unwrap_or(false),
                services,
                internal_services,
            };

            map.entry(uname).or_default().push(view);
        }
    }

    map
}

// ===== UTILITIES =====
fn extract_all_services(
    groups: &BTreeMap<String, Vec<DeviceView>>,
    svc_cfg: &ServiceConfig,
) -> Vec<ServiceView> {
    let mut quick = Vec::new();

    // Iterate through configured services
    for s in &svc_cfg.service {
        // Find which user owns this host
        if let Some((owner, devices)) = groups
            .iter()
            .find(|(_, devs)| devs.iter().any(|d| d.host_name == s.host))
        {
            // Find the matching device
            if let Some(dev) = devices.iter().find(|d| d.host_name == s.host && d.online) {
                quick.push(ServiceView {
                    name: s.name.clone(),
                    owner: owner.clone(),
                    url: s.url.clone(),
                });
            }
        }
    }

    quick
}

// ===== STRUCTS =====
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

#[derive(Clone)]
pub struct DeviceView {
    pub host_name: String,
    pub ip: String,
    pub os: String,
    pub online: bool,
    pub services: Vec<ServiceView>,
    pub internal_services: Vec<ServiceView>,
}

#[derive(Clone)]
pub struct ServiceView {
    pub name: String,
    pub owner: String,
    pub url: String,
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

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    pub service: Vec<ServiceEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceEntry {
    pub host: String,
    pub name: String,
    pub url: String,
}
