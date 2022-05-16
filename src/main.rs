#![feature(try_blocks)]

use std::collections::HashMap;
use std::mem;
use std::os::unix::prelude::AsRawFd;
use std::path::PathBuf;

use anyhow::anyhow;

use crate::events::Event;
use crate::raw_fd::AsyncRawFd;
use crate::dbus::bluez::{Device1Proxy, GattService1Proxy, GattCharacteristic1Proxy};

pub mod discovery;
pub mod events;
pub mod raw_fd;
pub mod util;
pub mod dbus;

// TODO: get rid of this
const SWITCH_TO_MANAGED: [u8; 10] = [0x02, 0xb0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

async fn set_hidraw_managed(device: &AsyncRawFd) -> anyhow::Result<()> {
    let write_bytes = device.write(&SWITCH_TO_MANAGED).await?;
    if write_bytes != SWITCH_TO_MANAGED.len() {
        Err(anyhow::anyhow!("failed to write to device"))
    } else {
        Ok(())
    }
}

async fn handle_hidraw(device: &AsyncRawFd) -> anyhow::Result<()> {
    set_hidraw_managed(device).await?;
    println!("setting device to managed mode");

    loop {
        let buf = device.read(64).await.unwrap();
        println!("read bytes: {}", to_hex(&buf));
        let maybe_parsed = Event::parse(&buf);
        match maybe_parsed {
            Ok(parsed) => {
                println!("parsed: {:?}", parsed);
                if let Event::WirelessAdapterStatus { connected } = parsed {
                    if connected {
                        println!("wireless adapter reports device connect, set managed");
                        set_hidraw_managed(device).await?;
                    }
                }
            }
            Err(err) => println!("parse error: {:?}", err),
        }
        println!("");
    }
}

async fn handle_bluetooth(path_str: String) -> anyhow::Result<()> {
    let connection = zbus::Connection::system().await?;
    let device = Device1Proxy::builder(&connection)
        .path(path_str.clone())?
        .build().await?;
    println!("bluetooth device name: {}", device.Name().await?);
    let connected = device.Connected().await?;
    if !connected {
        return Err(anyhow!("device not connected"));
    }

    let path = PathBuf::from(path_str);
    // FIXME: hardcoded stuff bad
    // uuid 00000001-0000-1000-8000-00805f9b34fb
    let mut send_char_path = path.clone();
    send_char_path.push("service002c/char002d");
    // uuid 00000003-0000-1000-8000-00805f9b34fb
    let mut recv_char_path = path.clone();
    recv_char_path.push("service002c/char0032");

    let send_char = GattCharacteristic1Proxy::builder(&connection)
        .path(send_char_path.to_str().unwrap())?
        .build().await?;
    let recv_char = GattCharacteristic1Proxy::builder(&connection)
        .path(recv_char_path.to_str().unwrap())?
        .build().await?;

    // grab notify fd
    let (notify_fd_dbus, mtu) = recv_char.AcquireNotify(HashMap::new()).await?;
    println!("receive mtu: {}", mtu);
    println!("notify fd: {}", notify_fd_dbus.as_raw_fd());
    let notify_fd = AsyncRawFd::from_fd(&notify_fd_dbus)?;
    mem::forget(notify_fd_dbus); // must forget to prevent close on drop

    // set device to managed mode
    send_char.WriteValue(SWITCH_TO_MANAGED.to_vec(), HashMap::new()).await?;

    loop {
        let buf = notify_fd.read(mtu as usize).await.unwrap();
        println!("read bytes: {}", to_hex(&buf));
        let maybe_parsed = Event::parse(&buf);
        match maybe_parsed {
            Ok(parsed) => {
                println!("parsed: {:?}", parsed);
            }
            Err(err) => println!("parse error: {:?}", err),
        }
        println!("");
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("usage: {} hidraw <hidraw device>", args[0]);
        println!("       {} bluetooth <dbus path of bluetooth device>", args[0]);
        return;
    }

    match args[1].as_str() {
        "hidraw" => {
            let device = AsyncRawFd::open(args[2].clone()).expect("cannot open hidraw device");
            handle_hidraw(&device).await.expect("handle_hidraw failed");
        },
        "bluetooth" => {
            handle_bluetooth(args[2].clone()).await.expect("handle_bluetooth failed");
        },
        _ => {
            eprintln!("invalid device type");
        }
    }
}
