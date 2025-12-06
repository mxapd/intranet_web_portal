use crate::{
    config::load_services_config,
    models::service::{ServiceConfig, ServiceView},
    models::tailscale::DeviceView,
    tailscale::{get_tailscale_status, group_by_user},
};
use askama::Template;
use axum::response::Html;
use std::collections::BTreeMap;

pub async fn serve() -> Html<String> {
    match (get_tailscale_status().await, load_services_config()) {
        (Ok(status), Ok(services_cfg)) => {
            let grouped = group_by_user(&status, &services_cfg);
            let quick_services = extract_all_services(&grouped, &services_cfg);
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

fn extract_all_services(
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

#[derive(Template)]
#[template(path = "index.html")]
struct PortalTemplate<'a> {
    users: &'a BTreeMap<String, Vec<DeviceView>>,
    quick_services: &'a Vec<ServiceView>,
}
