use crate::models::service::{ServiceConfig, ServiceView};
use crate::models::tailscale::{DeviceView, TailscaleStatus};
use std::collections::BTreeMap;
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command};

pub async fn get_tailscale_status() -> anyhow::Result<TailscaleStatus> {
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
                services,
                internal_services,
            };

            map.entry(uname).or_default().push(view);
        }
    }

    map
}
