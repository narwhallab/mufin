use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use std::thread;
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, CharPropFlags, WriteType, BDAddr};
use btleplug::platform::{Manager, PeripheralId};
use napi_derive::napi;
use log::info;
use simplelog::{TermLogger, Config};
use futures::stream::StreamExt;

#[napi]
pub fn init_logger() {
    TermLogger::init(log::LevelFilter::Trace, Config::default(), simplelog::TerminalMode::Stdout, simplelog::ColorChoice::Auto).unwrap();
}

#[napi]
pub async fn bluetooth(address: String, message: String) {
    write_peripheral(&address, &message.as_bytes()).await.expect("Error");
}

pub async fn get_peripheral(address: &str) -> Result<impl Peripheral, Box<dyn Error>> {
    let manager = Manager::new().await?;

    let central = manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("No adapters are available now...");    // Fetch first adapter

    central
        .start_scan(ScanFilter::default())
        .await
        .expect("Can't scan BLE adapter for connected devices...");

    info!("Starting scan on {}...", central.adapter_info().await?);
        
    thread::sleep(Duration::from_secs(5)); // Wait until the scan is done

    let peripheral = central.peripheral(&PeripheralId::from(BDAddr::from_str(address).unwrap())).await?;

    Ok(peripheral)
}

#[napi]
pub async fn bluetooth_read(address: String) {
    read_peripheral(&address).await.expect("Error");
}

pub async fn read_peripheral(address: &str) -> Result<(), Box<dyn Error>> {
    let peripheral = get_peripheral(address).await?;
    
    let properties = peripheral.properties().await?;
    let is_connected = peripheral.is_connected().await?;
    let local_name = properties
        .unwrap()
        .local_name
        .unwrap_or(String::from("Unknown"));

    if !is_connected {
        info!("Connecting to peripheral {}...", &local_name);
        peripheral.connect().await?;
    }

    let message = if is_connected { "succeeded" } else { "failed" };

    info!("Connecting to peripherial: {} has {}", &local_name, message);

    peripheral.discover_services().await?;
    for service in peripheral.services() {
        for characteristic in service.characteristics {
            if characteristic.properties.contains(CharPropFlags::NOTIFY) {
                println!("Subscribing to characteristic {:?}", characteristic.uuid);
                peripheral.subscribe(&characteristic).await?;
                let mut notification_stream =
                    peripheral.notifications().await?.take(4);
                while let Some(data) = notification_stream.next().await {
                    info!(
                        "Received data from {:?} [{:?}]: {:?}",
                        local_name, data.uuid, data.value
                    );
                }
            }
        }
}
Ok(())
}

pub async fn write_peripheral(address: &str, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let peripheral = get_peripheral(address).await?;
    
    let properties = peripheral.properties().await?;
    let is_connected = peripheral.is_connected().await?;
    let local_name = properties
        .unwrap()
        .local_name
        .unwrap_or(String::from("Unknown"));

    if !is_connected {
        info!("Connecting to peripheral {}...", &local_name);
        peripheral.connect().await?;
    }

    let message = if is_connected { "succeeded" } else { "failed" };

    info!("Connecting to peripherial: {} has {}", &local_name, message);

    peripheral.discover_services().await?;
    for service in peripheral.services() {
        for characteristic in service.characteristics {
            if characteristic.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) {
                peripheral.write(&characteristic, bytes, WriteType::WithoutResponse).await?;
            }
        }
    }

    if is_connected {
        info!("Disconnecting from peripheral {:?}...", &local_name);
        peripheral
            .disconnect()
            .await
            .expect("Error disconnecting from BLE peripheral");
    }
    Ok(())
}