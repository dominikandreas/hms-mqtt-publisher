use std::{thread, time::Duration};

use crate::{
    metric_collector::MetricCollector, mqtt_config::MqttConfig,
    protos::hoymiles::RealData::HMSStateResponse,
};

use log::{debug, warn};
use rumqttc::{Client, MqttOptions, QoS};

pub struct SimpleMqtt {
    client: Client,
}

impl SimpleMqtt {
    pub fn new(config: &MqttConfig) -> Self {
        let mut mqttoptions = MqttOptions::new(
            "hms800wt2-mqtt-publisher",
            &config.host,
            config.port.unwrap_or(1883),
        );
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        //parse the mqtt authentication options
        if let Some((username, password)) = match (&config.username, &config.password) {
            (None, None) => None,
            (None, Some(_)) => None,
            (Some(username), None) => Some((username.clone(), "".into())),
            (Some(username), Some(password)) => Some((username.clone(), password.clone())),
        } {
            mqttoptions.set_credentials(username, password);
        }

        let (client, mut connection) = Client::new(mqttoptions, 10);

        thread::spawn(move || {
            // keep polling the event loop to make sure outgoing messages get sent
            for _ in connection.iter() {}
        });

        Self { client }
    }
}

impl MetricCollector for SimpleMqtt {
    fn publish(&mut self, hms_state: &HMSStateResponse) {
        debug!("{hms_state}");

        let pv_current_power = hms_state.pv_current_power as f32 / 10.;
        let pv_daily_yield = hms_state.pv_daily_yield;

        self.client
            .subscribe("hms800wt2/pv_current_power", QoS::AtMostOnce)
            .unwrap();
        match self.client.publish(
            "hms800wt2/pv_current_power",
            QoS::AtMostOnce,
            true,
            pv_current_power.to_string(),
        ) {
            Ok(_) => {}
            Err(e) => warn!("mqtt error: {e}"),
        }
        match self.client.publish(
            "hms800wt2/pv_daily_yield",
            QoS::AtMostOnce,
            true,
            pv_daily_yield.to_string(),
        ) {
            Ok(_) => {}
            Err(e) => warn!("mqtt error: {e}"),
        }
    }
}