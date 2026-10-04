mod heatshrink_tools;

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::sleep;
use tokio_serial::SerialPortBuilderExt;

static BAD_APPLE: &[u8] = include_bytes!("../output.bin");

#[tokio::main]
async fn main() {
    let port_name = "/dev/cu.usbmodem421";
    println!("{}", BAD_APPLE.len());


    let mut port = tokio_serial::new(port_name, 115_200)
        .open_native_async()
        .unwrap();


    port.write_all(&[0xAA]).await.expect("Failed to handshake");
    let _  = port.flush().await;

    loop {
        let mut buf = [0u8; 10];
        // println!("Hello");
        match port.read_exact(&mut buf).await {
            Ok(_) => {
                let index = usize::from_be_bytes(buf[0..8].try_into().unwrap());
                // println!("{index}");
                let size = u16::from_be_bytes(buf[8..10].try_into().unwrap()) as usize;
                port.write_all(&BAD_APPLE[index..(index + size).min(BAD_APPLE.len())])
                    .await
                    .expect("Error on writing");
                // println!("{:?}", buf);
            }
            Err(e) => {
                println!("Hi :{e}");
                sleep(Duration::from_secs(1)).await;
            }
        };

    }
}
