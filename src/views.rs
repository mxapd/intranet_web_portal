use crate::models::service::ServiceConfig;
use crate::models::tailscale::TailscaleStatus;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct ServiceView {
    pub name: String,
    pub owner: String,
    pub url: String,
}

#[derive(Clone)]
pub struct DeviceView {
    pub host_name: String,
    pub ip: String,
    pub os: String,
    pub online: bool,
    pub last_seen: Option<String>,
    pub services: Vec<ServiceView>,
    pub internal_services: Vec<ServiceView>,
}

pub fn group_by_user(
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

            let services = svc_cfg
                .service
                .iter()
                .filter(|s| s.host == dev.host_name.clone().unwrap_or_default())
                .map(|s| {
                    let ip = dev
                        .ips
                        .as_ref()
                        .and_then(|v| v.iter().find(|ip| ip.starts_with("100.")))
                        .cloned()
                        .unwrap_or_else(|| "?".to_string());

                    let url = match (&s.url, s.port) {
                        (Some(u), _) => u.clone(),
                        (None, Some(port)) => format!("http://{}:{}", ip, port),
                        _ => format!("http://{}", ip),
                    };

                    ServiceView {
                        name: s.name.clone(),
                        owner: uname.clone(),
                        url,
                    }
                })
                .collect::<Vec<_>>();

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
                last_seen: dev
                    .last_seen
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                services,
                internal_services,
            };

            map.entry(uname).or_default().push(view);
        }
    }

    for devices in map.values_mut() {
        devices.sort_by(|a, b| {
            b.online
                .cmp(&a.online)
                .then_with(|| a.host_name.cmp(&b.host_name))
        });
    }

    map
}

pub fn extract_all_services(
    groups: &BTreeMap<String, Vec<DeviceView>>,
    svc_cfg: &ServiceConfig,
) -> Vec<ServiceView> {
    let mut quick = Vec::new();

    for s in &svc_cfg.service {
        if let Some((owner, devices)) = groups
            .iter()
            .find(|(_, devs)| devs.iter().any(|d| d.host_name == s.host))
        {
            if let Some(dev) = devices.iter().find(|d| d.host_name == s.host && d.online) {
                let url = match (&s.url, s.port) {
                    (Some(u), _) => u.clone(),
                    (None, Some(port)) => format!("http://{}:{}", dev.ip, port),
                    _ => format!("http://{}", dev.ip),
                };

                quick.push(ServiceView {
                    name: s.name.clone(),
                    owner: owner.clone(),
                    url,
                });
            }
        }
    }

    quick
}
