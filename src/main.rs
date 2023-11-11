// TODO: support CA33 command to take over metrics consumption
// TODO: support publishing to S-Miles cloud, too

mod inverter;
mod mqtt;
mod homeassistant_api;
mod protos;
mod logging;

use crate::inverter::Inverter;
use crate::mqtt::{MetricCollector, Mqtt};
use crate::homeassistant_api::publish_sensor_readings;

use std::thread;
use std::time::Duration;
use log::info;

use clap::{Parser, Subcommand};
use protos::hoymiles::RealData;


#[derive(Subcommand)]
enum Commands {
    /// Publish to Home Assistant HTTP API
    HomeAssistant {
        url: String,
        auth_token: String,
    },

    /// Publish to MQTT server
    Mqtt {
        mqtt_broker_host: String,
        mqtt_username: Option<String>,
        mqtt_password: Option<String>,
        #[clap(default_value = "1883")]
        mqtt_broker_port: u16,
    },
}

// Command line interface
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    inverter_host: String,

    #[clap(subcommand)]
    command: Commands,
}

static REQUEST_DELAY: u64 = 30_500;

async fn run() {
    logging::init_logger();

    let cli = Cli::parse();
    let mut inverter = Inverter::new(&cli.inverter_host);

    info!("Inverter host: {}", cli.inverter_host);

    match cli.command {
        Commands::HomeAssistant { url, auth_token } => {
            info!("Home Assistant: {}", url);
            info!("Bearer token: {}", auth_token);
            loop {
                if let Some(hms_state) = inverter.update_state() {
                    publish_sensor_readings(&hms_state, &url, &auth_token).await.unwrap();
                    thread::sleep(Duration::from_millis(REQUEST_DELAY));
                }
            }
        },
        Commands::Mqtt { mqtt_broker_host, mqtt_username, mqtt_password, mqtt_broker_port } => {
            info!("MQTT broker: {}", mqtt_broker_host);

            let mut mqtt = Mqtt::new(
                &mqtt_broker_host, 
                &mqtt_username.clone(),
                &mqtt_password.clone(),
                mqtt_broker_port
            );
                
            loop {
                if let Some(r) = inverter.update_state() {
                    mqtt.publish(&r);
                }
                thread::sleep(Duration::from_millis(REQUEST_DELAY));
            }
        }
    }
    
}

#[tokio::main]
async fn main() {
    run().await;
}