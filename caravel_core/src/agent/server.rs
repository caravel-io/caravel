use crate::agent::events::{Event, EventPayload, EventType};
use axum::http::StatusCode;
use axum::Json;

use crate::agent::capabilities::gather_capabilities;
use crate::agent::config::AgentConfig;
use crate::agent::dependencies::receive_dependencies;
use crate::agent::upload;
use crate::agent::util::generate_duration;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::time::sleep;

use axum::{routing::post, Router};

pub struct Agent {
    pub config_path: Option<PathBuf>,
}

impl Agent {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
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
            // This is the main agent server process
            tokio::spawn(serve(config.clone()));
        }

        // Also start the pull mode if a manifest was given
        if let Some(_manifest) = &config.manifest {
            // This may run alongside the main server and runs the given manifest
            // on a schedule. Usually you would disable the listen server if you're
            // running in pull mode.
            tokio::spawn(pull_loop(config.clone()));
        }

        loop {}
    }
}

// Transaction scheme between host and remote agent:
// 1. Host sends POST request to agent with ShipEvent
// 2. Agent receives POST request and sends back CapabilitiesResponse
// 3. Host receives CapabilitiesResponse and calculates dependencies
// 4. Host sends DependencyEvent (gzipped byte stream) to agent
// 5. Agent receives byte stream and writes to disk
// 6. Agent sends back DependencyResponse
// 7. Host receives DependencyResponse and sends ApplyManifestEvent
// 8. Agent receives ApplyManifestEvent and applies the manifest

async fn serve(config: AgentConfig) {
    let app = Router::new()
        .route("/", post(handler))
        .route("/upload", post(upload::handler));

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

pub async fn handler(Json(event): Json<Event>) -> (StatusCode, Json<Event>) {
    // eventually, we want to validate the connection somehow
    // patrick seems to know the dance
    match event.class {
        EventType::Query => {
            let capabilities_resp = match gather_capabilities().await {
                Ok(r) => r,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            Event::new(EventType::Error)
                                .message(format!("Failed to gather capabilities: {}", e)),
                        ),
                    )
                }
            };
            return (StatusCode::OK, Json(capabilities_resp));
        }

        EventType::Dependencies => {
            let assembled_response = match receive_dependencies().await {
                Ok(r) => r,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            Event::new(EventType::Error)
                                .message(format!("Failed to receive dependencies: {}", e)),
                        ),
                    )
                }
            };
            return (StatusCode::OK, Json(assembled_response));
        }

        EventType::ApplyManifest => {
            let _ = match event.payload {
                Some(p) => {
                    match &p {
                        EventPayload::Manifest(m) => {
                            println!("manifest: {:?}", m);
                        }
                    }
                    p
                }
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(
                            Event::new(EventType::Error)
                                .message("ApplyManifest events must have a payload".to_string()),
                        ),
                    )
                }
            };
            return (
                StatusCode::OK,
                Json(Event::new(EventType::ApplySuccess).message("applied manifest".to_string())),
            );
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(Event::new(EventType::Error).message("Invalid event type".to_string())),
            )
        }
    }
}
