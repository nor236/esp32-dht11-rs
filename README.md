# esp32-dht11-rs

`esp32-dht11-rs` is a `no_std` Rust driver for DHT11 temperature and humidity
sensors connected to ESP32-family microcontrollers. It uses `embedded-hal` for
delays and `esp-hal` for GPIO access.

## Chip features

The `esp32c3` feature is enabled by default. For another chip, disable default
features and select exactly one of `esp32`, `esp32c2`, `esp32c3`, `esp32c6`,
`esp32h2`, `esp32s2`, or `esp32s3`:

```toml
[dependencies]
esp32-dht11-rs = { version = "0.1.4", default-features = false, features = ["esp32s3"] }
```

## Usage

```rust
use embedded_hal::delay::DelayNs;
use esp32_dht11_rs::DHT11;
use esp_hal::delay::Delay;

let mut polling_delay = Delay::new();
let mut dht11 = DHT11::new(peripherals.GPIO2, Delay::new());

loop {
    match dht11.read() {
        Ok(reading) => log::info!(
            "DHT11 - temperature: {} °C, humidity: {} %",
            reading.temperature,
            reading.humidity
        ),
        Err(error) => log::error!("DHT11 read failed: {:?}", error),
    }

    // The DHT11 should not be sampled more than once per second.
    polling_delay.delay_ms(1_000);
}
```

## License

Licensed under the [MIT License](LICENSE).
