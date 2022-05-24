use ::termios::{os::linux::*, *};
use futures::ready;
use std::io::{self, Read, Write};
use std::os::unix::io::RawFd;
use std::os::unix::prelude::AsRawFd;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::unix::AsyncFd;
use tokio::io::AsyncWrite;

fn setup_fd(fd: RawFd) -> io::Result<()> {
    let mut termios = Termios::from_fd(fd)?;
    termios.c_cflag = B115200 | CS8 | CLOCAL | CREAD;
    termios.c_iflag |= IGNPAR;
    termios.c_iflag &= !(IXON | IXOFF | IXANY); // no flow control
    termios.c_oflag &= !OPOST; // raw output mode
    termios.c_lflag &= !(ICANON | ECHO | ECHOE | ISIG);
    termios.c_cc[VMIN] = 0; // require at least this many chars
    termios.c_cc[VTIME] = 0; // interchar timeout

    tcsetattr(fd, TCSANOW, &termios)?;
    tcflush(fd, TCIOFLUSH)?;

    Ok(())
}

pub struct AsyncSerial {
    inner: AsyncFd<std::fs::File>,
}

impl AsyncSerial {
    pub fn new(fd: std::fs::File) -> io::Result<Self> {
        setup_fd(fd.as_raw_fd())?;

        Ok(Self {
            inner: AsyncFd::new(fd)?,
        })
    }

    pub async fn read(&self, out: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.inner.readable().await?;

            match guard.try_io(|inner| inner.get_ref().read(out)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for AsyncSerial {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            let mut guard = ready!(self.inner.poll_write_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().write(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.inner.get_ref().flush()?;
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        todo!();
    }
}
