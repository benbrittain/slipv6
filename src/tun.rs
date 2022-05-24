use anyhow::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::raw::{c_char, c_short};
use std::os::unix::prelude::AsRawFd;

nix::ioctl_write_int!(tun_set_iff, b'T', 202);

pub const IFF_TUN: c_short = 0x0001;
pub const IFF_NO_PI: c_short = 0x1000;

const INTERFACE_NAME_SIZE: usize = 16;
const IFREQ_UNION_SIZE: usize = 24;

#[repr(C)]
pub struct IfReq {
    pub interface_name: [c_char; INTERFACE_NAME_SIZE],
    pub union: IfReqUnion,
}

impl IfReq {
    pub fn new(name: &str) -> Self {
        // manufacture an interface name of the right size (leaving a \n)
        let mut interface_name = [0; INTERFACE_NAME_SIZE];
        for (idx, byte) in name.bytes().enumerate().take(INTERFACE_NAME_SIZE - 1) {
            interface_name[idx] = byte as c_char;
        }

        IfReq {
            interface_name,
            union: IfReqUnion {
                data: [0; IFREQ_UNION_SIZE],
            },
        }
    }
}

#[repr(C)]
pub union IfReqUnion {
    pub data: [u8; IFREQ_UNION_SIZE],
    pub flags: c_short,
}

#[derive(Debug)]
pub struct Tun {
    pub fd: File,
}

impl Tun {
    pub fn new(name: &str) -> Result<Self, Error> {
        let fd = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;

        let mut req = IfReq::new(name);
        req.union.flags = IFF_TUN | IFF_NO_PI;

        unsafe {
            tun_set_iff(fd.as_raw_fd(), &mut req as *mut _ as u64)?;
        }
        Ok(Tun { fd })
    }

    /// Write to the TUN interface.
    #[allow(dead_code)]
    pub fn write(&self, buf: &mut [u8]) -> Result<usize, io::Error> {
        (&self.fd).write(buf)
    }

    /// Blocking Read from the TUN interface.
    #[allow(dead_code)]
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, io::Error> {
        (&self.fd).read(buf)
    }
}
