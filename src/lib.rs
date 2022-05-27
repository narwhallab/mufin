use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use std::thread;
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, CharPropFlags, WriteType, BDAddr};
use btleplug::platform::{Manager, PeripheralId, Adapter};
use napi_derive::napi;
use log::info;
use simplelog::{TermLogger, Config};
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use tokio::runtime::Runtime;

lazy_static! {
    pub static ref CENTRAL: Adapter = Runtime::new().unwrap().block_on(async {
        get_adapter().await
    });
}

#[napi]
pub fn init_logger() {
    TermLogger::init(log::LevelFilter::Trace, Config::default(), simplelog::TerminalMode::Stdout, simplelog::ColorChoice::Auto).unwrap();
}

#[napi]
pub async fn bluetooth(address: String, message: String) {
    write_peripheral(&address, &message.as_bytes()).await.expect("Error");
}

pub async fn get_adapter() -> Adapter {
    let manager = Manager::new().await.expect("Could not fetch manager");

    let adapter = manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("No adapters are available now...");    // Fetch first adapter

    adapter
}

#[napi]
pub async fn scan() {
    CENTRAL
        .start_scan(ScanFilter::default())
        .await
        .expect("Can't scan BLE adapter for connected devices...");

    thread::sleep(Duration::from_secs(5)); // Wait until the scan is done
}

#[napi]
pub async fn get_peripheral(address: String) {
    get_peripheral_internal(&address).await.expect("Error");
}

pub async fn get_peripheral_internal(address: &str) -> Result<impl Peripheral, Box<dyn Error>> {
    let peripheral = CENTRAL.peripheral(&PeripheralId::from(BDAddr::from_str(address).unwrap())).await?;
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
    Ok(peripheral)
}

#[napi]
pub async fn bluetooth_read(address: String) {
    read_peripheral(&address).await.expect("Error");
}

#[napi]
pub async fn disconnect(address: String) {
    let peripheral = get_peripheral_internal(&address).await.expect("Could not get peripheral");

    if peripheral.is_connected().await.expect("Could not get connection status") {
        info!("Disconnecting from peripheral {:?}...", &address);
        peripheral
            .disconnect()
            .await
            .expect("Error disconnecting from BLE peripheral");
    }
}

pub async fn read_peripheral(address: &str) -> Result<(), Box<dyn Error>> {
    let peripheral = get_peripheral_internal(address).await?;

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
                        "Received data from <somewhere> [{:?}]: {:?}",
                        data.uuid, data.value
                    );
                }
            }
        }
}
Ok(())
}

pub async fn write_peripheral(address: &str, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let peripheral = get_peripheral_internal(address).await?;
    
    peripheral.discover_services().await?;
    for service in peripheral.services() {
        for characteristic in service.characteristics {
            if characteristic.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) {
                peripheral.write(&characteristic, bytes, WriteType::WithoutResponse).await?;
            }
        }
    }
    Ok(())
}