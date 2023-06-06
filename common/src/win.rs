use crate::serve::{ServableListener, ServableStream};
use crate::{ReadFrom, WriteTo};
use async_trait::async_trait;
use std::ffi::{OsStr, OsString};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};
use tokio::sync::RwLock;
use tokio::time;

pub struct ServableNamedPipeServer {
    inner: RwLock<Option<NamedPipeServer>>,
    address: OsString,
}

#[async_trait]
impl<S: Into<OsString> + Send + 'static> ServableListener<S> for ServableNamedPipeServer {
    type Stream = NamedPipeServer;
    type SocketAddr = ();

    async fn bind(socket: S) -> io::Result<Self> {
        let address = socket.into();
        let this = ServerOptions::new()
            .first_pipe_instance(true)
            .create(address.clone())?;
        Ok(Self {
            inner: RwLock::new(Some(this)),
            address,
        })
    }

    async fn accept(&self) -> io::Result<(Self::Stream, Self::SocketAddr)> {
        self.inner.read().await.unwrap().connect().await?;
        let mut inner = self.inner.write().await.take().unwrap();
        *self.inner.write().await = Some(ServerOptions::new().create(&self.address)?);

        Ok((inner, ()))
    }
}

#[async_trait]
impl<S: Send + 'static> ServableStream<S> for NamedPipeServer {
    type OwnedReadHalf = NamedPipeServerOwnedReadHalf;
    type OwnedWriteHalf = NamedPipeServerOwnedWriteHalf;
    type ReadHalf<'a> = NamedPipeServerReadHalf<'a> where Self: 'a;
    type WriteHalf<'a> = NamedPipeServerWriteHalf<'a> where Self: 'a;

    async fn connect(_: S) -> io::Result<Self> {
        panic!("not a NamedPipeClient")
    }

    fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
        let this = Arc::new(self);

        (
            NamedPipeServerOwnedReadHalf(Arc::clone(&this)),
            NamedPipeServerOwnedWriteHalf(this),
        )
    }

    fn split(&mut self) -> (Self::ReadHalf<'_>, Self::WriteHalf<'_>) {
        (
            NamedPipeServerReadHalf(self),
            NamedPipeServerWriteHalf(self),
        )
    }
}

#[allow(non_snake_case)]
macro_rules! impl_ReadFrom_for_OwnedReadHalf {
    ($ty:ty) => {
        #[async_trait]
        impl ReadFrom for $ty {
            async fn readable(&self) -> io::Result<()> {
                self.0.readable().await
            }

            fn try_read(&self, buf: &mut [u8]) -> io::Result<usize> {
                self.0.try_read(buf)
            }
        }
    };
}

#[allow(non_snake_case)]
macro_rules! impl_WriteTo_for_OwnedWriteHalf {
    ($ty:ty) => {
        #[async_trait]
        impl WriteTo for $ty {
            async fn writable(&self) -> io::Result<()> {
                self.0.writable().await
            }

            fn try_write(&self, buf: &[u8]) -> io::Result<usize> {
                self.0.try_write(buf)
            }
        }
    };
}

pub struct NamedPipeServerOwnedReadHalf(Arc<NamedPipeServer>);
impl_ReadFrom_for_OwnedReadHalf!(NamedPipeServerOwnedReadHalf);

pub struct NamedPipeServerOwnedWriteHalf(Arc<NamedPipeServer>);
impl_WriteTo_for_OwnedWriteHalf!(NamedPipeServerOwnedWriteHalf);

pub struct NamedPipeServerReadHalf<'a>(&'a NamedPipeServer);
impl_ReadFrom_for!(lt NamedPipeServerReadHalf);

pub struct NamedPipeServerWriteHalf<'a>(&'a NamedPipeServer);
impl_WriteTo_for!(lt NamedPipeServerWriteHalf);

#[async_trait]
impl<S: AsRef<OsStr> + Send + 'static> ServableStream<S> for NamedPipeClient {
    type OwnedReadHalf = NamedPipeClientOwnedReadHalf;
    type OwnedWriteHalf = NamedPipeClientOwnedWriteHalf;
    type ReadHalf<'a> = NamedPipeClientReadHalf<'a> where Self: 'a;
    type WriteHalf<'a> = NamedPipeClientWriteHalf<'a> where Self: 'a;

    async fn connect(socket: S) -> io::Result<Self> {
        const ERROR_PIPE_BUSY: i32 = 231;

        let address = socket.as_ref();

        loop {
            match ClientOptions::new().open(address) {
                Ok(client) => break Ok(client),
                Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                    time::sleep(Duration::from_millis(50)).await
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
        let this = Arc::new(self);

        (
            NamedPipeClientOwnedReadHalf(Arc::clone(&this)),
            NamedPipeClientOwnedWriteHalf(this),
        )
    }

    fn split(&mut self) -> (Self::ReadHalf<'_>, Self::WriteHalf<'_>) {
        (
            NamedPipeClientReadHalf(self),
            NamedPipeClientWriteHalf(self),
        )
    }
}

pub struct NamedPipeClientOwnedReadHalf(Arc<NamedPipeClient>);
impl_ReadFrom_for_OwnedReadHalf!(NamedPipeClientOwnedReadHalf);

pub struct NamedPipeClientOwnedWriteHalf(Arc<NamedPipeClient>);
impl_WriteTo_for_OwnedWriteHalf!(NamedPipeClientOwnedWriteHalf);

pub struct NamedPipeClientReadHalf<'a>(&'a NamedPipeClient);
impl_ReadFrom_for!(lt NamedPipeClientReadHalf);

pub struct NamedPipeClientWriteHalf<'a>(&'a NamedPipeClient);
impl_WriteTo_for!(lt NamedPipeClientWriteHalf);
