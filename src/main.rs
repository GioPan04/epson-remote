use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS},
};

use crate::{config::CONFIG, wifi::wifi};

mod config;
mod wifi;

fn main() -> Result<()> {
    // https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = CONFIG;

    log::info!("Booting");

    let peripheals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let _wifi = wifi(
        config.wifi_ssid,
        config.wifi_pass,
        peripheals.modem,
        sysloop,
    )?;

    let broker_url = if !config.mqtt_pass.is_empty() {
        format!(
            "mqtt://{}:{}@{}:{}",
            config.mqtt_user, config.mqtt_pass, config.mqtt_host, config.mqtt_port
        )
    } else {
        format!("mqtt://{}:{}", config.mqtt_host, config.mqtt_port)
    };

    log::info!("Connecting to {}", broker_url);

    let mqtt_config = MqttClientConfiguration::default();

    let (mut client, _) =
        EspMqttClient::new(&broker_url, &mqtt_config).expect("Couldn't connect to mqtt broker");

    client
        .publish(
            "bedroom/projector/available",
            QoS::AtLeastOnce,
            true,
            "online".as_bytes(),
        )
        .expect("Couldn't send online message");

    Ok(())
}
