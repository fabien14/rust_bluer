use bluer::{Adapter, AdapterEvent, Address, AddressType, Device};
use bluer::agent::{Agent, RequestPinCode, RequestPasskey, ReqResult};
use futures::{pin_mut, stream::SelectAll, StreamExt};

use bluer::rfcomm::{ SocketAddr, Stream, Socket };
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::time::Duration;
use tokio::time;


async fn query_device(device: &Device) -> bluer::Result<()> {
    println!("    Address type:       {}", device.address_type().await?);
    println!("    Name:               {:?}", device.name().await?);
    println!("    Icon:               {:?}", device.icon().await?);
    println!("    Class:              {:?}", device.class().await?);
    println!("    UUIDs:              {:?}", device.uuids().await?.unwrap_or_default());
    println!("    Paired:             {:?}", device.is_paired().await?);
    println!("    Connected:          {:?}", device.is_connected().await?);
    println!("    Trusted:            {:?}", device.is_trusted().await?);
    println!("    Modalias:           {:?}", device.modalias().await?);
    println!("    RSSI:               {:?}", device.rssi().await?);
    println!("    TX power:           {:?}", device.tx_power().await?);
    println!("    Manufacturer data:  {:?}", device.manufacturer_data().await?);
    println!("    Service data:       {:?}", device.service_data().await?);
    Ok(())
}

async fn query_all_device_properties(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
    let device = adapter.device(addr)?;
    let props = device.all_properties().await?;
    for prop in props {
        println!("    {:?}", &prop);
    }
    Ok(())
}

async fn query_infos_device(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
    let device = adapter.device(addr)?;
    let name = device.name().await?;
    println!("    Name:               {:?}", &name);
    if name.as_deref() == Some("LED-BEDROOM") {
        println!("TTTTRRRROOUUUUUVVVVVEEEEEEEEE");
        {
            let sock_addr = SocketAddr::new(addr, 1);
            
            let socket_session_result = Socket::new();
            let socket_session = match socket_session_result {
                Ok(sock) => sock,
                Err(error) => panic!("Problem socket new : {:?}", error),
            };

            let socket_stream = socket_session.connect(sock_addr).await.expect("connection failed");
            println!("{:?}", socket_stream);

            //query_device(&adapter, addr).await;
            query_all_device_properties(&adapter, addr).await;
            println!("    Device provides our service!");

            let (mut rh, mut wh) = socket_stream.into_split();
            wh.write_all(b"1234").await.expect("write failed");

            println!("    Send end");
        }
    }
    Ok(())
}

async fn device_connect(device: &Device) -> bluer::Result<Stream> {

    let device_adresse = device.address();
    let sock_addr = SocketAddr::new(device_adresse, 1);

    println!("    Connecting...");
    let mut retries = 2;
    loop {
        match Stream::connect(sock_addr).await {
            Ok(stream_socket) => return Ok(stream_socket),
            Err(err) if retries > 0 => {
                println!("    Connect error: {}", &err);
                retries -= 1;
            }
            Err(err) => return Err(err.into()),
        }
    }
}

const SCAN_DURATION: Duration = Duration::from_secs(5);

async fn request_pin_code(_req: RequestPinCode) -> ReqResult<String> {
    println!("call pin");
    Ok("1234".into())
}
async fn request_passkey(_req: RequestPasskey) -> ReqResult<u32> {
    println!("passkey");
    Ok(1234)
}

#[tokio::main]
async fn main() -> bluer::Result<()> {

    let session = bluer::Session::new().await?;
    let agent = Agent {
        request_pin_code: Some(Box::new(|req| Box::pin(request_pin_code(req)))),
        request_passkey: Some(Box::new(|req| Box::pin(request_passkey(req)))),
        ..Default::default()
    };
    let handle_agent = session.register_agent(agent).await;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    println!("Advertising on Bluetooth adapter {} with address {}", adapter.name(), adapter.address().await?);

    {
        let discover = adapter.discover_devices().await?;
        time::sleep(SCAN_DURATION).await;

        let device_addresses_return = adapter.device_addresses().await;
        let device_addresses = match device_addresses_return {
            Ok(device_addresses) => device_addresses,
            Err(error) => panic!("Qdresse found : {:?}", error),
        };

        let mut target_device : Option<Device> = None;
        for device_addresse in device_addresses {
            println!("{:?}", device_addresse);

            let device = adapter.device(device_addresse)?;
            let name = device.name().await?;

            if name.as_deref() == Some("LED-BEDROOM") {
                println!("Device found: {device_addresse}");
                target_device = Some(device);
            }
        }

        let stream_device = match target_device {
            Some(device) => {
                query_device(&device).await;
                device.pair().await;
                let stream = device_connect(&device).await.unwrap();
                println!("{:?}", stream);
                Ok(stream)
            },
            _ => Err({}),
        };

        match stream_device {
            Ok(stre) => {
                println!("{:?}", stre);
                println!("oulalala le stream est ouvert");
                let (mut rh, mut wh) = stre.into_split();
                wh.write_all(b"1234").await.expect("write failed");
                time::sleep(SCAN_DURATION).await;
                wh.write_all(b"1234").await.expect("write failed");
            },
            _ => (),
        }
    }

    Ok(())
}
