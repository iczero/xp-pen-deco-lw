#![feature(try_blocks)]

use bytes::BytesMut;
use tokio::fs::{read_link, read_dir, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;
use std::sync::Arc;

use crate::events::Event;
use crate::hidraw::HidrawDevice;

pub mod hidraw;
pub mod util;
pub mod events;

async fn sys_derp() {
    let hidraw_path = Path::new("/sys/class/hidraw");
    let mut hidraw_dir = read_dir(hidraw_path).await.unwrap();
    todo!();
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
}

async fn read_events(device: &HidrawDevice) {
    loop {
        let buf = device.read(64).await.unwrap();
        println!("read bytes: {}", to_hex(&buf));
        let maybe_parsed = Event::parse(&buf);
        match maybe_parsed {
            Ok(parsed) => println!("parsed: {:?}", parsed),
            Err(err) => println!("parse error: {:?}", err),
        }
        println!("");
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <hidraw device>", args[0]);
        return;
    }

    let hidraw_device = HidrawDevice::open(args[1].clone()).expect("cannot open hidraw device");
    let arc = Arc::new(hidraw_device);
    let arc_cloned = arc.clone();
    let read_stuff = tokio::spawn(async move {
        read_events(&arc_cloned).await
    });
    
    let write_bytes = arc.write(
        &[0x02, 0xb0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ).await.expect("write failed");
    assert_eq!(write_bytes, 10, "eh");
    println!("write succeeded");
    read_stuff.await.unwrap();
}
