extern crate hub_sdk;
extern crate geeny_api;
extern crate uuid;
extern crate rpassword;

use hub_sdk::{HubSDK, HubSDKConfig};
use hub_sdk::services::PartialThingMessage;

use geeny_api::ThingsApi;
use geeny_api::ConnectApi;
use geeny_api::models::ThingRequest;

use uuid::Uuid;

use std::path::PathBuf;

use std::thread;
use std::time::Duration;

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
    
    // Log in
    
    let username = String::from("chris@geeny.io");
    println!("Password?");
    let password = rpassword::read_password()
        .expect("Unexpected error.");
    println!("Password provided.\n");
    
    hub_sdk.login(&username, &password)
        .expect("Failed to log in.");
    
    println!("{} logged in.\n\r", username);
    
    // Create a Thing
    
    let name = "alpha";
    let serial_number = "omega";
    let thing_type = Uuid::parse_str("877827cc-0c78-4e55-80fe-2941479c681a")
        .unwrap();
    
    let thing = ThingRequest {
        name: String::from(name),
        serial_number: String::from(serial_number),
        thing_type: thing_type,
    };
    
    // TODO: Better result management?
    // Is it possible to check if Thing already exists?
    match hub_sdk.create_thing(thing) {
        Ok(_) => println!("Thing created.\n\r"),
        Err(_) => println!("No new Thing created.\n\r"),
    }
    
    // Send messages to the Cloud
    
    let topic = "human_message";
    let contents = [
        "Hello, world!",
        "Hallo, Welt!",
        "¡Hola, mundo!",
    ];
    let period = Duration::from_secs(1);
    
    println!("Messages sent:");
    
    thread::sleep(period);
    let messages = [
        PartialThingMessage {
            topic: String::from(topic),
            msg: String::from(contents[0]),
        },
    ];
    match hub_sdk.send_messages(&serial_number, &messages) {
        Ok(_) => println!("{}", messages[0].msg),
        Err(_) => println!("Failed to send messages to the Cloud."),
    }
    
    thread::sleep(period);
    let messages = [
        PartialThingMessage {
            topic: String::from(topic),
            msg: String::from(contents[1]),
        },
    ];
    match hub_sdk.send_messages(&serial_number, &messages) {
        Ok(_) => println!("{}", messages[0].msg),
        Err(_) => println!("Failed to send messages to the Cloud."),
    }
    
    thread::sleep(period);
    let messages = [
        PartialThingMessage {
            topic: String::from(topic),
            msg: String::from(contents[2]),
        },
    ];
    match hub_sdk.send_messages(&serial_number, &messages) {
        Ok(_) => println!("{}", messages[0].msg),
        Err(_) => println!("Failed to send messages to the Cloud."),
    }
    
    // Finish
    
    thread::sleep(Duration::from_secs(1));
    println!("\n\r*** SIMULATION FINISHED ***\n\r");
}
