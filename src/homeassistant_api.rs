use reqwest::Client;
use crate::protos::hoymiles::RealData::HMSStateResponse;
use serde_json::json;


pub(crate) async fn publish_sensor_readings(hms_state: &HMSStateResponse, url: &str, auth_token: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let pv_current_power = hms_state.pv_current_power as f32 / 10.;
    let pv_daily_yield = hms_state.pv_daily_yield;

    publish_state(
        &client, &url, &auth_token, "sensor.hms_800w_t2_hms_pv_power", 
        &pv_current_power.to_string(),
        &json!({
            "unit_of_measurement": "W",
            "device_class": "power",
            "state_class": "measurement",
        }),

    ).await?;
    publish_state(
        &client, &url, &auth_token, "sensor.hms_800w_t2_hms_pv_daily_yield", 
        &pv_daily_yield.to_string(),
        &json!({
            "unit_of_measurement": "kWh",
            "device_class": "energy",
            "state_class": "total_increasing",
        }),

    ).await?;

    Ok(())
}


async fn publish_state(
    client: &Client, url: &str, auth_token: &str, entity_id: &str, 
    state: &str, 
    //attribute document
    attributes: &serde_json::Value,
) -> Result<(), reqwest::Error> {
    let state_url = format!("{}/api/states/{}", url, entity_id);
    let response = client.post(&state_url)
        .bearer_auth(auth_token)
        .json(&json!({
            "state": state,
            "attributes": attributes,
        }))
        .send()
        .await?;

    if response.status().is_success() {
        println!("Sensor reading published successfully");
    } else {
        eprintln!("Failed to publish sensor reading: {:?}", response.text().await?);
    }

    Ok(())
}