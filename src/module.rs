use crate::cli::Runnable;
use std::path::PathBuf;

pub struct CreateModule {
    pub destination: PathBuf,
}

impl Runnable for CreateModule {
    fn run(&self) {
        println!("Creating new module at: {:?}", self.destination);
    }
}

pub struct ValidateModule {
    pub path: PathBuf,
}

impl Runnable for ValidateModule {
    fn run(&self) {
        println!("Validating module at: {:?}", self.path);
    }
}

// wip
use crate::event::{Event, EventType, QueryType};
use crate::examplemodulefile::File;
use crate::manifest::Manifest;
use anyhow::Result;
use std::process::exit;

#[allow(dead_code)]
fn run_module() -> Result<()> {
    // Read the input from only arg
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        let error_event = Event {
            class: EventType::Error("Invalid number of arguments".to_string()),
        };
        error_event.write_to_stderr()?;
        exit(1)
    }

    // The first arg is the program name, we need the second one
    let data = &args[1];

    // print a sample apply event to stdout
    // let apply_event = Event {
    //     class: EventType::Apply(Manifest {
    //         resources: vec![Box::new(File {
    //             // name: "test.txt".to_string(),
    //             // content: "Hello, World!".to_string(),
    //         })],
    //     }),
    // };
    // println!("{}", serde_json::to_string(&apply_event)?);

    // data is a valid UTF-8 string
    // now we need to deserialize it
    let recv_event: Event = match serde_json::from_str(&data) {
        Ok(event) => event,
        Err(_) => {
            let error_event = Event {
                class: EventType::Error("Invalid event".to_string()),
            };
            error_event.write_to_stderr()?;
            exit(1)
        }
    };

    // Now we have a valid event, do something with it
    let reply_event = match recv_event.class {
        EventType::Query(QueryType::Health) => Event {
            class: EventType::Reply("OK".to_string()),
        },

        EventType::Query(QueryType::Features) => Event {
            class: EventType::Reply("Feature 1, Feature 2".to_string()),
        },

        EventType::Apply(manifest) => {
            manifest.resources.iter().for_each(|r| {
                if let Err(e) = r.apply() {
                    let error_event = Event {
                        class: EventType::Error(format!("Error applying resource: {}", e)),
                    };
                    error_event.write_to_stderr().unwrap();
                    exit(1);
                }
            });
            Event {
                class: EventType::Reply("Applied Manifest".to_string()),
            }
        }

        _ => Event {
            class: EventType::Error("Invalid event type".to_string()),
        },
    };

    reply_event.write_to_stdout()?;

    Ok(())
}
