use std::{error::Error, str::FromStr};
use btleplug::{platform::{Adapter, Manager, PeripheralId}, api::{Manager as _, Central as _, CharPropFlags, WriteType, Peripheral, BDAddr}};
use futures::StreamExt;
use log::info;

use crate::CENTRAL;

pub async fn get_adapter() -> Adapter {
    let manager = Manager::new().await.expect("Could not fetch manager");

    manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("No adapters are available now...")
}

pub async fn connect_peripheral(address: &str) -> Result<impl Peripheral, Box<dyn Error>> {
    let peripheral = CENTRAL.get()
        .await
        .peripheral(&PeripheralId::from(BDAddr::from_str(address).unwrap())).await?;
    
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

    info!("Connection to peripherial: {} has {}", &local_name, message);
    Ok(peripheral)
}

pub async fn read_peripheral(address: &str) -> Result<(), Box<dyn Error>> {
    let peripheral = connect_peripheral(address).await?;
    peripheral.discover_services().await?;

    for service in peripheral.services() {
        for characteristic in service.characteristics {
            if characteristic.properties.contains(CharPropFlags::NOTIFY) {
                info!("Subscribing to characteristic {:?}", characteristic.uuid);
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
    let peripheral = connect_peripheral(address).await?;
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