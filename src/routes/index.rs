use crate::{
    config::load_services_config,
    tailscale::get_tailscale_status,
    views::{DeviceView, ServiceView, extract_all_services, group_by_user},
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

#[derive(Template)]
#[template(path = "index.html")]
struct PortalTemplate<'a> {
    users: &'a BTreeMap<String, Vec<DeviceView>>,
    quick_services: &'a Vec<ServiceView>,
}
