mod bluetooth;

use std::time::Duration;
use std::thread;
use async_once::AsyncOnce;
use btleplug::api::{Central, ScanFilter, Peripheral};
use btleplug::platform::Adapter;
use napi_derive::napi;
use log::info;
use simplelog::{TermLogger, Config};
use lazy_static::lazy_static;
use bluetooth::{get_adapter, connect_peripheral, read_peripheral, write_peripheral};

lazy_static! {
    pub static ref CENTRAL: AsyncOnce<Adapter> = AsyncOnce::new(async {
        get_adapter().await
    });
}

#[napi]
pub fn init_logger() {
    TermLogger::init(log::LevelFilter::Info, Config::default(), simplelog::TerminalMode::Stdout, simplelog::ColorChoice::Auto).unwrap();
}

#[napi]
pub async fn scan_bluetooth() {
    CENTRAL.get().await
        .start_scan(ScanFilter::default())
        .await
        .expect("Can't scan BLE adapter for connected devices...");

    thread::sleep(Duration::from_secs(5)); // Wait until the scan is done
}

#[napi]
pub async fn connect_bluetooth(address: String) {
    connect_peripheral(&address).await.expect("Error");
}

#[napi]
pub async fn read_bluetooth(address: String) {
    read_peripheral(&address).await.expect("Error");
}

#[napi]
pub async fn write_bluetooth(address: String, message: String) {
    write_peripheral(&address, &message.as_bytes()).await.expect("Error");
}

#[napi]
pub async fn disconnect_bluetooth(address: String) {
    let peripheral = connect_peripheral(&address).await.expect("Could not get peripheral");

    if peripheral.is_connected().await.expect("Could not get connection status") {
        info!("Disconnecting from peripheral {:?}...", &address);
        peripheral
            .disconnect()
            .await
            .expect("Error disconnecting from BLE peripheral");
    }
}