use crate::{
    config::{load_services_config, save_services_config},
    models::service::{AddServiceForm, RemoveServiceForm, ServiceEntry},
    tailscale::get_tailscale_status,
};
use askama::Template;
use axum::{
    extract::Form,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};

pub async fn serve() -> Html<String> {
    match (get_tailscale_status().await, load_services_config()) {
        (Ok(status), Ok(services_cfg)) => {
            let mut hosts = Vec::new();
            if let Some(self_dev) = &status.self_device {
                if let Some(hostname) = &self_dev.host_name {
                    hosts.push(hostname.clone());
                }
            }
            if let Some(peers) = &status.peers {
                for dev in peers.values() {
                    if let Some(hostname) = &dev.host_name {
                        if dev.online.unwrap_or(false) {
                            hosts.push(hostname.clone());
                        }
                    }
                }
            }
            hosts.sort();
            hosts.dedup();

            let template = ManageTemplate {
                services: &services_cfg.service,
                available_hosts: &hosts,
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

pub async fn add_service(Form(input): Form<AddServiceForm>) -> impl IntoResponse {
    match load_services_config() {
        Ok(mut config) => {
            // Create new service entry
            let new_service = ServiceEntry {
                host: input.host,
                name: input.name,
                port: if input.port > 0 {
                    Some(input.port)
                } else {
                    None
                },
                url: if input.url.is_empty() {
                    None
                } else {
                    Some(input.url)
                },
            };

            config.service.push(new_service);

            // Save back to file
            match save_services_config(&config) {
                Ok(_) => Redirect::to("/manage").into_response(),
                Err(e) => {
                    eprintln!("Error saving config: {e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save config").into_response()
                }
            }
        }
        Err(e) => {
            eprintln!("Error loading config: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load config").into_response()
        }
    }
}

pub async fn remove_service(Form(input): Form<RemoveServiceForm>) -> impl IntoResponse {
    match load_services_config() {
        Ok(mut config) => {
            // Remove service at the given index
            if input.index < config.service.len() {
                config.service.remove(input.index);

                match save_services_config(&config) {
                    Ok(_) => Redirect::to("/manage").into_response(),
                    Err(e) => {
                        eprintln!("Error saving config: {e}");
                        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save config").into_response()
                    }
                }
            } else {
                (StatusCode::BAD_REQUEST, "Invalid service index").into_response()
            }
        }
        Err(e) => {
            eprintln!("Error loading config: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load config").into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "manage.html")]
struct ManageTemplate<'a> {
    services: &'a Vec<ServiceEntry>,
    available_hosts: &'a Vec<String>,
}
