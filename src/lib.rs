use std::error::Error;
use std::time::Duration;
use std::thread;
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, CharPropFlags, WriteType};
use btleplug::platform::Manager;
use node_bindgen::derive::node_bindgen;

#[node_bindgen]
async fn bluetooth() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;

    let central = manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("No adapters are available now...");

    println!("Starting scan on {}...", central.adapter_info().await?);

    central
        .start_scan(ScanFilter::default())
        .await
        .expect("Can't scan BLE adapter for connected devices...");

    thread::sleep(Duration::from_secs(10)); // Wait until the scan is done

    let peripherals = central.peripherals().await?;
    if peripherals.is_empty() {
        eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
    } else {
        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await?;
            let is_connected = peripheral.is_connected().await?;
            let local_name = properties
                .unwrap()
                .local_name
                .unwrap_or(String::from("(peripheral name unknown)"));
            if !local_name.contains("HMSoft") { // TODO This part
                continue;
            }
            println!("Peripheral {:?} is connected: {:?}", local_name, is_connected);
            if !is_connected {
                println!("Connecting to peripheral {:?}...", &local_name);
                if let Err(err) = peripheral.connect().await {
                    eprintln!("Error connecting to peripheral, skipping: {}", err);
                    continue;
                }
            }
            let is_connected = peripheral.is_connected().await?;
            println!("Now connected ({:?}) to peripheral {:?}...", is_connected, &local_name);
            peripheral.discover_services().await?;
            println!("Discover peripheral {:?} services...", &local_name);
            for service in peripheral.services() {
                println!("Service UUID {}, primary: {}", service.uuid, service.primary);
                for characteristic in service.characteristics {
                    if characteristic.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) {
                        peripheral.write(&characteristic, &"on".as_bytes(), WriteType::WithoutResponse).await?;
                    }
                }
            }
            if is_connected {
                println!("Disconnecting from peripheral {:?}...", &local_name);
                peripheral
                    .disconnect()
                    .await
                    .expect("Error disconnecting from BLE peripheral");
            }
        }
    }
    Ok(())
}