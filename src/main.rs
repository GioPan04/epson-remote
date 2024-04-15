use anyhow::Result;
use esp_idf_hal::{
    gpio,
    peripherals::Peripherals,
    uart::{self, UartDriver},
    units::Hertz,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    mqtt::client::{EspMqttClient, EventPayload, MqttClientConfiguration, QoS},
};
use std::fmt::Write;
use std::sync::{mpsc, Arc, Mutex};

use crate::{config::CONFIG, wifi::wifi};

mod config;
mod models;
mod wifi;

fn main() -> Result<()> {
    // https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = CONFIG;

    log::info!("Booting");

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let mut uart = UartDriver::new(
        peripherals.uart1,
        peripherals.pins.gpio17,
        peripherals.pins.gpio16,
        Option::<gpio::Gpio0>::None,
        Option::<gpio::Gpio1>::None,
        &uart::config::Config::new().baudrate(Hertz(115200)),
    )?;

    let _wifi = wifi(
        config.wifi_ssid,
        config.wifi_pass,
        peripherals.modem,
        sysloop,
    )?;

    let broker_url = config.mqtt_uri();
    log::info!("Connecting to {}", broker_url);
    let mqtt_config = MqttClientConfiguration::default();

    let (client, mut connection) = EspMqttClient::new(&broker_url, &mqtt_config)?;

    let client = Arc::new(Mutex::new(client));

    std::thread::scope(|s| {
        let listener = Arc::clone(&client);

        let (tx, rx) = mpsc::channel::<models::response::Response>();

        std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(s, move || {
                let client = listener;

                while let Ok(msg) = rx.recv() {
                    let payload: &str = msg.into();
                    let mut client = client.lock().unwrap();
                    client
                        .enqueue(
                            "bedroom/projector/switch",
                            QoS::AtLeastOnce,
                            false,
                            payload.as_bytes(),
                        )
                        .unwrap();
                }
            })
            .unwrap();

        std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(s, move || {
                while let Ok(event) = connection.next() {
                    log::info!("{}", event.payload());

                    match event.payload() {
                        EventPayload::Received {
                            topic: Some(topic),
                            data,
                            ..
                        } => {
                            if topic == "bedroom/projector/switch/set" {
                                if data == "ON".as_bytes() {
                                    // log::info!("{:?}", "PWR ON".to_ascii_uppercase().as_bytes());
                                    // uart.write("PWR ON".to_ascii_uppercase().as_bytes())
                                    //     .unwrap();
                                    writeln!(uart, "PWR ON").unwrap();
                                    tx.send(models::response::Response::TurnOn).unwrap()
                                } else {
                                    uart.write("PWR OFF".to_ascii_uppercase().as_bytes())
                                        .unwrap();
                                    tx.send(models::response::Response::TurnOff).unwrap()
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
            .unwrap();

        {
            let mut client = client.lock().unwrap();

            client
                .subscribe("bedroom/projector/switch/set", QoS::AtLeastOnce)
                .unwrap();

            client
                .enqueue(
                    "bedroom/projector/available",
                    QoS::AtMostOnce,
                    false,
                    "online".as_bytes(),
                )
                .unwrap();

            log::info!("Awake message sent");
        }
    });

    Ok(())
}
