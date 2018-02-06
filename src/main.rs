extern crate hub_sdk;
extern crate geeny_api;
extern crate uuid;
extern crate rpassword;
extern crate serde;
extern crate serde_json;
extern crate chrono;

#[macro_use]
extern crate serde_derive;

use hub_sdk::{HubSDK, HubSDKConfig};
use hub_sdk::services::PartialThingMessage;

use geeny_api::{ThingsApi, ConnectApi};
use geeny_api::models::ThingRequest;

use uuid::Uuid;

use chrono::prelude::*;

use std::path::PathBuf;
use std::io::{self, Read, Write, BufWriter};
use std::thread;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use std::fs::File;

// TODO: Extract logic from main() and into a function that returns an
// error/result code.
fn main() {
    // START
    
    println!("\n\r*** GEENY DEVICE SIMULATOR STARTED ***\n\r");
    println!("[{}] Application started.\n\r", timestamp());
    
    // Create SDK configuration structure
    
    let sdk_cfg = HubSDKConfig {
        api: ThingsApi::default(),
        connect_api: ConnectApi::default(),
        element_file: PathBuf::from("./geeny/elements.json"),
        geeny_creds_file: PathBuf::from("./geeny/user_credentials.json"),
        mqtt_cert_path: PathBuf::from("./geeny/mqtt_certificates"),
        mqtt_host: String::from("mqtt.geeny.io"),
        mqtt_port: 8883,
    };
    
    // Create Hub SDK instance
    
    let hub_sdk = HubSDK::new(sdk_cfg);
    
    // Load configuration (from the configuration file)
    
    // TODO: Introduce a JSON schema for validation?
    let mut config_file = File::open("./config/config.json")
        .expect("Configuration file not found.");
    let mut config = String::new();
    config_file.read_to_string(&mut config)
        .expect("Unexpected error.");
    let config = config.trim();
    // Only for debugging purposes
    //println!("Configuration:\n\r{}\n\r", config);
    
    // TODO: Relocate structure definitions?
    #[derive(Deserialize, Debug, Clone)]
    struct Simulation {
        thing_name: String,
        thing_sn: String,
        thing_type: Uuid,
        msg_topic: String,
        msg_content: String,
        period_ms: u64,
        duration_ms: u64,
    }
    
    #[derive(Deserialize, Debug, Clone)]
    struct Config {
        user: String,
        sims: Vec<Simulation>,
    }
    
    let config: Config = serde_json::from_str(config).unwrap();
    // Only for debugging purposes
    //println!("Loaded configuration:\n\r{:#?}\n\r", config);
    
    // Check user's credentials; authenticate if necessary
    
    println!("Checking user's credentials...");
    
    let (username, valid) = hub_sdk.check_token()
        .expect("Failed to retrieve authentication information.");
    if (username == config.user) && (valid == true) {
        println!("User \"{}\" is already logged in.\n\r", config.user);
    } else {
        println!("Username: {}", config.user);
        print!("Password: ");
        io::stdout().flush().unwrap();
        let password = rpassword::read_password()
            .expect("Unexpected error.");
        let password = password.trim();
    
        println!();
        
        hub_sdk.login(&config.user, password)
            .expect("Failed to log in.");
        println!("User \"{}\" logged in.\n\r", config.user);
    }
    
    // Spawn one thread per simulation; execute all of them in parallel
    
    let (tx, rx) = channel();
    let len = config.sims.len();
    
    for i in 0..len {
        let sim = config.sims[i].clone();
        let sdk = hub_sdk.clone();
        let tx = tx.clone();
        
        thread::spawn(move || {
            // Register Thing to the Cloud
            
            let thing = ThingRequest {
                name: sim.thing_name.clone(),
                serial_number: sim.thing_sn.clone(),
                thing_type: sim.thing_type,
            };
            
            let mut writer = BufWriter::new(io::stdout());
            writer.write(b"Attempting to create Thing...\n\r").unwrap();
            // TODO: Better result management?
            // Is it possible to check if the desired Thing already exists?
            match sdk.create_thing(thing) {
                Ok(_) => {
                    write!(writer, "Thing \"{:}\" created.\n\r\n\r",
                        sim.thing_name).unwrap();
                }
                Err(_) => {
                    write!(writer, "Thing \"{:}\" already exists.\n\r\n\r",
                        sim.thing_name).unwrap();
                }
            }
            writer.flush().unwrap();
            
            // Send messages to the Cloud
            
            let messages = [
                PartialThingMessage {
                    topic: sim.msg_topic,
                    msg: sim.msg_content,
                },
            ];
            let period = Duration::from_millis(sim.period_ms);
            let duration = Duration::from_millis(sim.duration_ms);
            
            let start = Instant::now();
            while start.elapsed() <= duration {
                thread::sleep(period);
                match sdk.send_messages(&sim.thing_sn, &messages) {
                    Ok(_) =>
                        println!("[{}] Message sent: {}",
                            timestamp(), messages[0].msg),
                    Err(_) =>
                        println!("Failed to send messages to the Cloud."),
                }
            }
            
            // Retrieve messages received from the Cloud (if any)
            
            let mut messages : Vec<PartialThingMessage> = Vec::new();
            
            writer.write(b"\n\rRetrieving messages from the Cloud...\n\r")
                .unwrap();
            // TODO: Better result management?
            match sdk.receive_messages(&sim.thing_sn) {
                Ok(msgs) => {
                    messages = msgs;
                    let word = if messages.len() == 1 {
                        "message"
                    } else {
                        "messages"
                    };
                    write!(writer, "[{}] {} {} received.\n\r",
                        timestamp(), messages.len(), word).unwrap();
                }
                Err(_) => {
                    writer.write(b"Failed to retrieve received messages.")
                        .unwrap();
                }
            }
            
            for element in &messages {
                write!(writer, "Message received: {}\n\r", element.msg)
                    .unwrap();
            }
            
            writer.write(b"\n\r").unwrap();
            writer.flush().unwrap();
            
            // Send "end" signal to the parent thread
            
            tx.send(i).unwrap();
        });
    }
    
    // Wait for "end" signals from all child threads
    
    for _ in 0..len {
        rx.recv().unwrap();
    }
    
    // FINISH
    
    thread::sleep(Duration::from_secs(2));
    println!("[{}] Application finished.", timestamp());
    println!("\n\r*** GEENY DEVICE SIMULATOR FINISHED ***\n\r");
}

fn timestamp() -> String {
    Utc::now().to_rfc3339()
}
