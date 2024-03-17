use crate::events::{Event, EventType};
use anyhow::{Context, Result};
use rand::{self, thread_rng, Rng};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

use crate::manifest;
use axum::http::StatusCode;
use axum::{routing::post, Json, Router};
use serde::Deserialize;

pub struct Agent {
    pub config: Option<PathBuf>,
}

/*
Example toml config:

listen_port = '8080'
listen_address = '0.0.0.0'
interval = '30'
manifest = '/path/to/manifest.lua' # or 'https://url/to/manifest.lua'
splay = '0'

*/

#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
    // Listen mode
    pub listen_port: Option<u64>,
    pub listen_address: Option<String>,
    pub disable_listen: Option<bool>,

    // Pull mode
    pub interval: Option<u64>,
    pub splay: Option<u64>,
    pub manifest: Option<String>,
}

impl AgentConfig {
    // Default values should be set here
    pub fn new() -> AgentConfig {
        AgentConfig {
            listen_port: Some(1336),
            listen_address: Some("0.0.0.0".to_string()),
            disable_listen: Some(false),

            interval: Some(30),
            splay: Some(0),
            manifest: None,
        }
    }

    // This function merges in a config that was brought in via TOML
    pub fn merge_with(&mut self, other: &AgentConfig) {
        if let Some(listen_port) = other.listen_port {
            self.listen_port = Some(listen_port);
        }
        if let Some(listen_address) = &other.listen_address {
            self.listen_address = Some(listen_address.clone());
        }
        if let Some(disable_listen) = &other.disable_listen {
            self.disable_listen = Some(disable_listen.clone());
        }

        if let Some(interval) = other.interval {
            self.interval = Some(interval);
        }
        if let Some(splay) = other.splay {
            self.splay = Some(splay);
        }
        if let Some(manifest) = &other.manifest {
            self.manifest = Some(manifest.clone());
        }
    }

    // This one applies environment variables after all other configs
    pub fn merge_environment(&mut self) {
        if let Ok(port) = std::env::var("CARAVEL_AGENT_PORT") {
            self.listen_port = Some(port.parse().unwrap());
        }
        if let Ok(address) = std::env::var("CARAVEL_AGENT_ADDRESS") {
            self.listen_address = Some(address);
        }
        if let Ok(dl) = std::env::var("CARAVEL_AGENT_DISABLE_LISTEN") {
            let valids = vec!["true", "false", "1", "0"];
            if valids.contains(&dl.as_str()) {
                if dl == "true" || dl == "1" {
                    self.disable_listen = Some(true);
                } else {
                    self.disable_listen = Some(false);
                }
            }
        }

        if let Ok(interval) = std::env::var("CARAVEL_AGENT_INTERVAL") {
            self.interval = Some(interval.parse().unwrap());
        }
        if let Ok(splay) = std::env::var("CARAVEL_AGENT_SPLAY") {
            self.splay = Some(splay.parse().unwrap());
        }
        if let Ok(manifest) = std::env::var("CARAVEL_AGENT_MANIFEST") {
            self.manifest = Some(manifest);
        }
    }
}

impl Agent {
    pub async fn run(&self) -> Result<()> {
        // First check if config was provided
        // If it was, merge it with the default options

        let mut config = AgentConfig::new();

        // Merge config file if it was provided
        if let Some(c) = &self.config {
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

fn generate_duration(interval: u64, splay: u64) -> Duration {
    let random_splay = thread_rng().gen_range(0 - splay..=splay);
    let random_duration = Duration::from_secs(interval * 60 + random_splay);
    // just in case they set some wonky values for interval and splay
    if random_duration.as_secs() <= 0 {
        return Duration::from_secs(60);
    }
    random_duration
}
