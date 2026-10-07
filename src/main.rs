mod heatshrink_tools;

use std::time::Duration;
use heatshrink::encoder::HeatshrinkEncoder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::sleep;
use tokio_serial::SerialPortBuilderExt;
use crate::heatshrink_tools::run_encode;

static BAD_APPLE: &[u8] = include_bytes!("../output.bin");

#[tokio::main]
async fn main() {
    let input: &[u8] = include_bytes!("../bad_apple_full.bin");
    let mut compressed = [0u8; 8 * 1024 * 500];
    let len = compressed.len();

    let mut encoder: HeatshrinkEncoder<10, 4, 2048> = HeatshrinkEncoder::new();

    let encoded = run_encode(&mut encoder, input, &mut compressed).unwrap();
    let total_out = encoded.len();

    println!("{total_out}");


    loop {
        let port_name = "/dev/cu.usbmodem421";
        println!("{}", encoded.len());

        match tokio_serial::new(port_name, 115_200).open_native_async() {
            Ok(mut port) => {
                let mut handshake = [0u8; 9];
                handshake[0] = 0xAA;
                handshake[1..9].copy_from_slice(&total_out.to_be_bytes());
                port.write_all(&handshake).await.expect("Failed to handshake");
                let _ = port.flush().await;

                loop {
                    let mut buf = [0u8; 10];
                    match port.read_exact(&mut buf).await {
                        Ok(_) => {
                            let index = usize::from_be_bytes(buf[0..8].try_into().unwrap());
                            let size = u16::from_be_bytes(buf[8..10].try_into().unwrap()) as usize;
                            if size != 64 {
                                println!("index: {index}, size: {size}, len: {}", (&encoded[index..(index + size)]).len());
                            }
                            if index >= encoded.len() {
                                println!("Index bigger than len! {index}");
                                let zero_buf = vec![0u8; size];
                                port.write_all(&zero_buf).await.unwrap();
                                port.flush().await.unwrap();
                                continue;
                            }
                            port.write_all(&encoded[index..(index + size)])
                                .await
                                .expect("Error on writing");
                            port.flush().await.unwrap();
                        }
                        Err(e) => {
                            println!("Hi :{e}");
                            sleep(Duration::from_secs(1)).await;
                            break;
                        }
                    };
                }
            }
            Err(_) => {
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
    }
}
