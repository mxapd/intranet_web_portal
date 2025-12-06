use crate::models::tailscale::TailscaleStatus;
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
