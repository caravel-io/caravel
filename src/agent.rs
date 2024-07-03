use crate::config::AgentConfig;
use crate::events::{Event, EventType};
use anyhow::{Context, Result};
use rand::{self, thread_rng, Rng};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

use crate::manifest;
use axum::http::StatusCode;
use axum::{routing::post, Json, Router};

pub struct Agent {
    pub config_path: Option<PathBuf>,
}

impl Agent {
    pub async fn run(&self) -> Result<()> {
        // First check if config was provided
        // If it was, merge it with the default options

        let mut config = AgentConfig::new();

        // Merge config file if it was provided
        if let Some(c) = &self.config_path {
            let config_str = std::fs::read_to_string(c).context("Failed to read config file")?;
            let provided_config: AgentConfig = toml::from_str(&config_str)?;
            config.merge_with(&provided_config);
        }

        // Merge with environment variables
        config.merge_environment();

        println!("Running agent with config: {:?}", config);

        // If listen is disabled, we don't need to start the server
        if config.disable_listen != Some(true) {
            tokio::spawn(serve(config.clone()));
        }

        // Also start the pull mode if a manifest was given
        if let Some(_manifest) = &config.manifest {
            tokio::spawn(pull_loop(config.clone()));
        }

        loop {}
    }
}

async fn serve(config: AgentConfig) {
    let app = Router::new().route("/", post(receive));

    let addr = format!(
        "{}:{}",
        config.listen_address.unwrap(),
        config.listen_port.unwrap()
    );
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    match axum::serve(listener, app).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    };
}

async fn pull_loop(config: AgentConfig) {
    loop {
        match pull(config.clone()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to pull manifest: {}", e);
            }
        };
        let splayed_sleep = generate_duration(config.interval.unwrap(), config.splay.unwrap());
        sleep(splayed_sleep).await;
    }
}

fn pull(config: AgentConfig) -> Result<()> {
    println!("Pulling manifest: {:?}", config.manifest);
    Ok(())
}

async fn receive(Json(event): Json<Event>) -> (StatusCode, Json<Event>) {
    // eventually, we want to validate the connection somehow
    // patrick seems to know the dance
    if event.class != EventType::ApplyManifest {
        return (
            StatusCode::BAD_REQUEST,
            Json(Event::new(
                EventType::Error,
                Some("Only Apply events are accepted".to_string()),
            )),
        );
    }
    if event.message.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(Event::new(
                EventType::Error,
                Some("Apply events must have a message".to_string()),
            )),
        );
    }
    let m = match serde_json::from_str(&event.message.unwrap()) {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(Event::new(
                    EventType::Error,
                    Some(format!("Failed to parse message: {}", e)),
                )),
            )
        }
    };
    match manifest::apply(m) {
        Ok(_) => {
            return (
                StatusCode::CREATED,
                Json(Event::new(
                    EventType::ApplySuccess,
                    Some("Manifest applied successfully".to_string()),
                )),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Event::new(
                    EventType::ApplyFailure,
                    Some(format!("Failed to apply manifest: {}", e)),
                )),
            )
        }
    }
}

/// Generate a random duration based on the interval and splay
/// The duration will be between interval - splay and interval + splay
/// If the interval is 0, it will default to 60 seconds
fn generate_duration(interval: u64, splay: u64) -> Duration {
    let random_splay: i64 = thread_rng().gen_range(0 - splay as i64..=splay as i64);
    let interval_seconds = (interval * 60) as i64;
    let duration = Duration::from_secs((interval_seconds + random_splay) as u64);
    // just in case they set some wonky values for interval and splay
    if duration.as_secs() <= 0 {
        return Duration::from_secs(60);
    }
    duration
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_duration() {
        let interval = 30; // 30 minutes
        let splay = 10; // 10 seconds
                        // it should be between 29:50 and 30:10 or 1790 and 1810 seconds
        let d = generate_duration(interval, splay);
        println!("Duration: {:?}", d);
        assert!(d.as_secs() >= 1790 && d.as_secs() <= 1810);
    }
}
