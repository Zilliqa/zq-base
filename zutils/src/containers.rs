use crate::commands::CommandBuilder;
use anyhow::{anyhow, Result};
use std::time::Duration;
use tokio::time;

pub async fn is_container_running(container_name: &str) -> Result<bool> {
    let is_running = CommandBuilder::new()
        .cmd(
            "docker",
            &vec!["inspect", "-f", "{{.State.Running}}", container_name],
        )
        .run_logged()
        .await?;
    Ok(is_running.status_code == 0)
}

pub async fn wait_for_container_running(
    container_name: &str,
    wait_ms: u64,
    poll_interval_ms: u64,
) -> Result<bool> {
    for _ in 0..(wait_ms / poll_interval_ms) {
        if is_container_status_running(container_name).await? {
            return Ok(true);
        }
        time::sleep(Duration::from_millis(poll_interval_ms)).await;
    }

    Ok(false)
}

pub async fn wait_for_container_stopped(
    container_name: &str,
    wait_ms: u64,
    poll_interval_ms: u64,
) -> Result<bool> {
    for _ in 0..(wait_ms / poll_interval_ms) {
        if !is_container_status_running(container_name).await? {
            return Ok(true);
        }
        time::sleep(Duration::from_millis(poll_interval_ms)).await;
    }

    Ok(false)
}

pub async fn is_container_status_running(container_name: &str) -> Result<bool> {
    print!("💬 Check if container {0} is running", container_name);
    let check_args = vec![
        "container",
        "inspect",
        "-f",
        "{{.State.Status}}",
        container_name,
    ];
    let result = CommandBuilder::new()
        .cmd("docker", &check_args)
        .silent()
        .ignore_failures()
        .run()
        .await?;
    let output = result.sanitise_stdout()?;
    Ok(if result.success && output == "running" {
        println!("💫 Yes");
        true
    } else {
        println!("💫 No");
        false
    })
}

pub async fn kill_container(container_name: &str) -> Result<()> {
    let _ = CommandBuilder::new()
        .cmd("docker", &vec!["kill", container_name])
        .ignore_failures()
        .run()
        .await?;
    let _ = CommandBuilder::new()
        .cmd("docker", &vec!["rm", container_name])
        .ignore_failures()
        .run()
        .await?;
    Ok(())
}

pub struct ParsedImage {
    pub base_url: String,
    // image_name: String,
    pub version: String,
}

impl ParsedImage {
    pub fn from_url(url: &str) -> Result<Self> {
        let mut path_parts = url.split('/').collect::<Vec<_>>();
        if path_parts.len() < 2 {
            return Err(anyhow!("Invalid URL format"));
        }

        let image_name_with_version = path_parts.pop().unwrap();
        let image_parts: Vec<&str> = image_name_with_version.split(':').collect();

        let image_name = image_parts
            .first()
            .ok_or_else(|| anyhow!("Missing image name"))?;
        let version = image_parts.get(1).unwrap_or(&"latest");

        let base_parts = path_parts.join("/");
        let base_url = format!("{}/{}", base_parts, image_name);

        Ok(ParsedImage {
            base_url,
            //image_name: image_name.to_string(),
            version: version.to_string(),
        })
    }
}
