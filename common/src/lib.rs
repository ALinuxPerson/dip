#[macro_use]
mod macros {
    macro_rules! read_exact_or_break {
        ($var:expr, $buf:expr, $error_message:expr) => {
            match $var.read_exact_or_break($buf).await {
               ControlFlow::Break(()) => return ControlFlow::Break(Ok(())),
               ControlFlow::Continue(Ok(())) => (),
               ControlFlow::Continue(Err(error)) => return ControlFlow::Break(Err(error).context($error_message)),
            }
        };
    }

    macro_rules! write_all_or_break {
        ($var:expr, $buf:expr, $error_message:expr) => {
            match $var.write_all_or_break($buf).await {
               ControlFlow::Break(()) => return ControlFlow::Break(Ok(())),
               ControlFlow::Continue(Ok(())) => (),
               ControlFlow::Continue(Err(error)) => return ControlFlow::Break(Err(error).context($error_message)),
            }
        };
    }
}

pub mod dirs;
pub mod serve;
pub mod utils {
    use std::future::Future;
    use std::process::ExitCode;

    pub async fn try_main<F: Future<Output = anyhow::Result<()>>>(main_fn: impl FnOnce() -> F) -> ExitCode {
        if let Err(error) = main_fn().await {
            tracing::error!("{error:#}");
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}

pub use serve::serve;
pub use dirs::dirs;
use std::{env, io};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use async_trait::async_trait;
use tokio::net::tcp::{
    OwnedReadHalf as OwnedTcpReadHalf,
    OwnedWriteHalf as OwnedTcpWriteHalf,
    ReadHalf as TcpReadHalf,
    WriteHalf as TcpWriteHalf,
};
use tokio::net::unix::{
    OwnedReadHalf as OwnedUnixReadHalf,
    OwnedWriteHalf as OwnedUnixWriteHalf,
    ReadHalf as UnixReadHalf,
    WriteHalf as UnixWriteHalf,
};
use anyhow::Context;

#[async_trait]
pub trait ReadFrom: Sync {
    async fn readable(&self) -> io::Result<()>;
    fn try_read(&self, buf: &mut [u8]) -> io::Result<usize>;

    fn try_read_exact(&self, mut buf: &mut [u8]) -> io::Result<()> {
        while !buf.is_empty() {
            match self.try_read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        if !buf.is_empty() {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
        } else {
            Ok(())
        }
    }

    async fn read_exact(&self, buf: &mut [u8]) -> io::Result<()> {
        let result = self.try_read_exact(buf);

        if let Err(io::ErrorKind::WouldBlock) = result.as_ref().map_err(|e| e.kind()) {
            self.readable().await?;
            self.read_exact(buf).await
        } else {
            result
        }
    }

    async fn read_exact_or_break(&self, buf: &mut [u8]) -> ControlFlow<(), io::Result<()>> {
        let result = self.read_exact(buf).await;

        if let Err(io::ErrorKind::UnexpectedEof) = result.as_ref().map_err(|e| e.kind()) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(result)
        }
    }
}

#[allow(non_snake_case)]
macro_rules! impl_ReadFrom_for {
    ($ty:ty) => {
        #[async_trait]
        impl ReadFrom for $ty {
            async fn readable(&self) -> io::Result<()> {
                <$ty>::readable(self).await
            }

            fn try_read(&self, buf: &mut [u8]) -> io::Result<usize> {
                <$ty>::try_read(self, buf)
            }
        }
    };
    
    (lt $ty:ident) => {
        #[async_trait]
        impl<'a> ReadFrom for $ty<'a> {
            async fn readable(&self) -> io::Result<()> {
                <$ty>::readable(self).await
            }

            fn try_read(&self, buf: &mut [u8]) -> io::Result<usize> {
                <$ty>::try_read(self, buf)
            }
        }
    };
}

impl_ReadFrom_for!(OwnedTcpReadHalf);
impl_ReadFrom_for!(OwnedUnixReadHalf);
impl_ReadFrom_for!(lt TcpReadHalf);
impl_ReadFrom_for!(lt UnixReadHalf);

#[async_trait]
pub trait WriteTo: Sync {
    async fn writable(&self) -> io::Result<()>;
    fn try_write(&self, buf: &[u8]) -> io::Result<usize>;

    fn try_write_all(&self, mut buf: &[u8]) -> io::Result<()> {
        while !buf.is_empty() {
            match self.try_write(buf) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    async fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        let result = self.try_write_all(buf);

        if let Err(io::ErrorKind::WouldBlock) = result.as_ref().map_err(|e| e.kind()) {
            self.writable().await?;
            self.write_all(buf).await
        } else {
            Ok(())
        }
    }

    async fn write_all_or_break(&self, buf: &[u8]) -> ControlFlow<(), io::Result<()>> {
        let result = self.write_all(buf).await;

        if let Err(io::ErrorKind::UnexpectedEof) = result.as_ref().map_err(|e| e.kind()) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(result)
        }
    }
}

#[allow(non_snake_case)]
macro_rules! impl_WriteTo_for {
    ($ty:ty) => {
        #[async_trait]
        impl WriteTo for $ty {
            async fn writable(&self) -> io::Result<()> {
                <$ty>::writable(self).await
            }

            fn try_write(&self, buf: &[u8]) -> io::Result<usize> {
                <$ty>::try_write(self, buf)
            }
        }
    };
    
    (lt $ty:ident) => {
        #[async_trait]
        impl<'a> WriteTo for $ty<'a> {
            async fn writable(&self) -> io::Result<()> {
                <$ty>::writable(self).await
            }

            fn try_write(&self, buf: &[u8]) -> io::Result<usize> {
                <$ty>::try_write(self, buf)
            }
        }
    };
}

impl_WriteTo_for!(OwnedTcpWriteHalf);
impl_WriteTo_for!(OwnedUnixWriteHalf);
impl_WriteTo_for!(lt TcpWriteHalf);
impl_WriteTo_for!(lt UnixWriteHalf);

pub fn find_socket(mut test_fn: impl FnMut(&Path) -> bool) -> Option<PathBuf> {
    let tmp_path = env::var_os("XDG_RUNTIME_DIR")
        .or_else(|| env::var_os("TMPDIR"))
        .or_else(|| env::var_os("TMP"))
        .or_else(|| env::var_os("TEMP"))
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new("/tmp").to_owned());

    (0..10)
        .map(|i| tmp_path.join(format!("discord-ipc-{i}")))
        .find(|socket_path| test_fn(socket_path))
}

#[tracing::instrument(skip_all)]
pub async fn read_from_then_write_to<R: ReadFrom, W: WriteTo>(read_from: &R, write_to: &W, read_from_name: &str, write_to_name: &str) -> ControlFlow<anyhow::Result<()>> {
    let mut header_buffer = [0; 8];
    read_exact_or_break!(read_from, &mut header_buffer, "failed to read header buffer");

    let header = u64::from_le_bytes(header_buffer);
    tracing::debug!(?header, "buffer");

    // length is the last 4 bytes of the header portion of the packet
    let length = (header >> 32) as u32;
    tracing::debug!(?length, "packet length");

    let mut json_content = vec![0; length as usize];
    read_exact_or_break!(read_from, &mut json_content, "failed to read json content");

    let mut packet = Vec::with_capacity(8 + length as usize);
    packet.extend(header_buffer);
    packet.extend(json_content);
    tracing::trace!("{read_from_name} -> {write_to_name}: \"{}\"", String::from_utf8_lossy(&packet));

    match write_to.write_all_or_break(&packet).await {
        ControlFlow::Break(()) => ControlFlow::Break(Ok(())),
        ControlFlow::Continue(Ok(())) => ControlFlow::Continue(()),
        ControlFlow::Continue(Err(error)) => ControlFlow::Break(Err(error).context("failed to write packet")),
    }
}
