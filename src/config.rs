use crate::models::service::ServiceConfig;

pub fn load_services_config() -> anyhow::Result<ServiceConfig> {
    let data = std::fs::read_to_string("config/services.toml")?;
    let config: ServiceConfig = toml::from_str(&data)?;
    Ok(config)
}

pub fn save_services_config(config: &ServiceConfig) -> anyhow::Result<()> {
    let toml_string = toml::to_string_pretty(config)?;
    std::fs::write("config/services.toml", toml_string)?;
    Ok(())
}
