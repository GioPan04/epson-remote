use esp_idf_sys::u_int32_t;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_pass: &'static str,

    #[default("test.mosquitto.org")]
    mqtt_host: &'static str,
    #[default(1883)]
    mqtt_port: u_int32_t,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
}
