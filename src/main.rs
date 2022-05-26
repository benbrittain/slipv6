#![deny(warnings)]

///! slipv6
///
/// Because sometimes you made a mistake...
use anyhow::Error;

use tokio::io::AsyncWriteExt;

use serial_line_ip as slip;

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

    eprintln!("slipv6 has has started");

    let mut to_host_buffer = [0; 4096];
    let mut to_device_buffer = [0; 4096];

    let mut slip_encode = [0; 4096];

    loop {
        tokio::select! {
            tun_bytes = tun.read(&mut to_device_buffer) => {
                let bytes_read = tun_bytes?;

                let mut slip_enc = slip::Encoder::new();
                let mut totals = slip_enc.encode(&to_device_buffer[..bytes_read], &mut slip_encode).unwrap();
                totals += slip_enc.finish(&mut slip_encode[totals.written..]).unwrap();
                serial.write(&slip_encode[..totals.written]).await?
            },
            serial_bytes = serial.read(&mut to_host_buffer) => {
                let bytes_read = serial_bytes?;
                eprintln!("Need to do this! {:?}", &to_host_buffer[..bytes_read]);
                // TODO transform the bytes here
                tun.write(&to_host_buffer[..bytes_read]).await?
            }
        };
    }
}
