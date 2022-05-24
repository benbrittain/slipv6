#![deny(warnings)]

///! slipv6
///
/// Because sometimes you made a mistake...
use anyhow::Error;

use tokio::io::AsyncWriteExt;

mod async_serial;
mod async_tun;
mod tun;

use async_serial::AsyncSerial;
use async_tun::AsyncTun;
use tun::Tun;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut tun = AsyncTun::new(Tun::new("slip")?)?;

    let serial_port = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/ttyUSB0")?;

    let mut serial = AsyncSerial::new(serial_port)?;

    let mut to_host_buffer = [0; 4096];
    let mut to_device_buffer = [0; 4096];
    loop {
        tokio::select! {
            tun_bytes = tun.read(&mut to_device_buffer) => {
                let bytes_read = tun_bytes?;
                // TODO transform the bytes here
                serial.write(&to_device_buffer[..bytes_read]).await?
            },
            serial_bytes = serial.read(&mut to_host_buffer) => {
                let bytes_read = serial_bytes?;
                // TODO transform the bytes here
                tun.write(&to_host_buffer[..bytes_read]).await?
            }
        };
    }
}
