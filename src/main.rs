extern crate hub_sdk;
extern crate geeny_api;
extern crate uuid;
extern crate rpassword;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use hub_sdk::{HubSDK, HubSDKConfig};
use hub_sdk::services::PartialThingMessage;

use geeny_api::{ThingsApi, ConnectApi};
use geeny_api::models::ThingRequest;

use uuid::Uuid;

use std::path::PathBuf;

use std::io;
use std::io::{Read, Write};

use std::thread;
use std::time::{Duration, Instant};

use std::fs::File;

fn main() {
    // Start
    
    println!("\n\r*** GEENY DEVICE SIMULATOR STARTED ***\n\r");
    
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
    
    #[derive(Deserialize, Debug)]
    struct Simulation {
        thing_name: String,
        thing_sn: String,
        thing_type: String,
        msg_topic: String,
        msg_content: String,
        period_ms: u64,
        duration_ms: u64,
    }
    
    #[derive(Deserialize, Debug)]
    struct Config {
        user: String,
        sims: Vec<Simulation>,
    }
    
    let config: Config = serde_json::from_str(config).unwrap();
    // Only for debugging purposes
    println!("Loaded configuration:\n\r{:#?}\n\r", config);
    
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
    
    // Register Thing to the Cloud
    
    let thing = ThingRequest {
        name: config.sims[0].thing_name.clone(),
        serial_number: config.sims[0].thing_sn.clone(),
        thing_type: Uuid::parse_str(&config.sims[0].thing_type).unwrap(),
    };
    // TODO: Better result management?
    // Is it possible to check if the desired Thing already exists?
    match hub_sdk.create_thing(thing) {
        Ok(_) => println!("Thing created.\n\r"),
        Err(_) => println!("No new Thing created.\n\r"),
    }
    
    // Send messages to the Cloud
    
    let messages = [
        PartialThingMessage {
            topic: config.sims[0].msg_topic.clone(),
            msg: config.sims[0].msg_content.clone(),
        },
    ];
    
    let period = Duration::from_millis(config.sims[0].period_ms);
    let duration = Duration::from_millis(config.sims[0].duration_ms);
    
    println!("Messages sent:");
    
    let start = Instant::now();
    
    while start.elapsed() <= duration {
        thread::sleep(period);
        match hub_sdk.send_messages(&config.sims[0].thing_sn, &messages) {
            Ok(_) => println!("{}", messages[0].msg),
            Err(_) => println!("Failed to send messages to the Cloud."),
        }
    }
    
    // Finish
    
    thread::sleep(Duration::from_secs(1));
    println!("\n\r*** SIMULATION FINISHED ***\n\r");
}
