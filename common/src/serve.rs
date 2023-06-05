#[cfg(unix)]
mod unix {
    use std::io;
    use std::path::Path;
    use async_trait::async_trait;
    use tokio::net::{unix, UnixListener, UnixStream};
    use tokio::net::unix::SocketAddr;
    use unix::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf};
    use crate::serve::{ServableListener, ServableStream};

    #[async_trait]
    impl<S: AsRef<Path> + Send + 'static> ServableListener<S> for UnixListener {
        type Stream = UnixStream;
        type SocketAddr = SocketAddr;

        async fn bind(socket: S) -> io::Result<Self> {
            UnixListener::bind(socket)
        }

        async fn accept(&self) -> io::Result<(Self::Stream, Self::SocketAddr)> {
            UnixListener::accept(self).await
        }
    }

    #[async_trait]
    impl<S: AsRef<Path> + Send + 'static> ServableStream<S> for UnixStream {
        type OwnedReadHalf = OwnedReadHalf;
        type OwnedWriteHalf = OwnedWriteHalf;
        type ReadHalf<'a> = ReadHalf<'a> where Self: 'a;
        type WriteHalf<'a> = WriteHalf<'a> where Self: 'a;

        async fn connect(socket: S) -> io::Result<Self> {
            UnixStream::connect(socket).await
        }

        fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
            UnixStream::into_split(self)
        }

        fn split(&mut self) -> (Self::ReadHalf<'_>, Self::WriteHalf<'_>) {
            UnixStream::split(self)
        }
    }
}

use crate::{ReadFrom, WriteTo};
use anyhow::Context;
use async_trait::async_trait;
use std::fmt::{Debug, Display};
use std::io;
use std::net::SocketAddr;
use std::path::Display as DisplayablePath;
use std::path::{Path, PathBuf};
use tokio::net::{tcp, TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;

#[async_trait]
pub trait ServableListener<S: Send + 'static>: Sized {
    type Stream: ServableStream<S>;
    type SocketAddr;

    async fn bind(socket: S) -> io::Result<Self>;
    async fn accept(&self) -> io::Result<(Self::Stream, Self::SocketAddr)>;
}

#[async_trait]
impl<S: ToSocketAddrs + Send + 'static> ServableListener<S> for TcpListener {
    type Stream = TcpStream;
    type SocketAddr = SocketAddr;

    async fn bind(socket: S) -> io::Result<Self> {
        TcpListener::bind(socket).await
    }

    async fn accept(&self) -> io::Result<(Self::Stream, Self::SocketAddr)> {
        TcpListener::accept(self).await
    }
}

#[async_trait]
pub trait ServableStream<S: Send + 'static>: Sized {
    type OwnedReadHalf: ReadFrom + Send + 'static;
    type OwnedWriteHalf: WriteTo + Send + 'static;
    type ReadHalf<'a>: ReadFrom + Send
    where
        Self: 'a;
    type WriteHalf<'a>: WriteTo + Send
    where
        Self: 'a;

    async fn connect(socket: S) -> io::Result<Self>;
    fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf);
    fn split(&mut self) -> (Self::ReadHalf<'_>, Self::WriteHalf<'_>);
}

#[async_trait]
impl<S: ToSocketAddrs + Send + 'static> ServableStream<S> for TcpStream {
    type OwnedReadHalf = tcp::OwnedReadHalf;
    type OwnedWriteHalf = tcp::OwnedWriteHalf;
    type ReadHalf<'a> = tcp::ReadHalf<'a> where Self: 'a;
    type WriteHalf<'a> = tcp::WriteHalf<'a> where Self: 'a;

    async fn connect(socket: S) -> io::Result<Self> {
        TcpStream::connect(socket).await
    }

    fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
        TcpStream::into_split(self)
    }

    fn split(&'_ mut self) -> (Self::ReadHalf<'_>, Self::WriteHalf<'_>) {
        TcpStream::split(self)
    }
}

pub trait Displayable {
    type Display<'d>: Display
    where
        Self: 'd;

    fn display(&self) -> Self::Display<'_>;
}

impl Displayable for PathBuf {
    type Display<'d> = DisplayablePath<'d> where Self: 'd;

    fn display(&self) -> Self::Display<'_> {
        Path::display(self)
    }
}

#[allow(non_snake_case)]
macro_rules! impl_Displayable {
    ($ty:ty) => {
        impl Displayable for $ty {
            type Display<'d> = &'d $ty;

            fn display(&self) -> Self::Display<'_> {
                self
            }
        }
    };
}

impl_Displayable!(SocketAddr);

pub type OnStreamConnectFail = Box<dyn FnMut(&io::Error)>;

#[derive(Default)]
pub struct ServeHooks {
    pub on_stream_connect_fail: Option<OnStreamConnectFail>,
}

impl ServeHooks {
    pub fn on_stream_connect_fail(mut self, run: impl FnMut(&io::Error) + 'static) -> Self {
        self.on_stream_connect_fail = Some(Box::new(run));
        self
    }
}

#[tracing::instrument(skip_all)]
async fn read_then_send_worker<R, W>(
    read_from: &R,
    write_to: &W,
    read_from_name: &str,
    write_to_name: &str,
    finished_receiver: mpsc::Sender<()>,
)
where
    R: ReadFrom,
    W: WriteTo,
{
    tracing::trace!("{read_from_name}->{write_to_name} started");

    loop {
        if crate::read_from_then_write_to(
            read_from,
            write_to,
            read_from_name,
            write_to_name,
        )
            .await
            .is_break()
        {
            tracing::debug!(
                        "finished reading from {read_from_name} and sending to {write_to_name}"
                    );

            finished_receiver.send(()).await.unwrap();
            break;
        }
    }
}

pub async fn serve<L, S, LS, SS>(
    listener_bind_to: LS,
    stream_connect_to: SS,
    new_client_name: &'static str,
    stream_name: &'static str,
    mut hooks: ServeHooks,
) -> anyhow::Result<()>
where
    LS: Displayable + Send + 'static,
    L: ServableListener<LS>,
    L::SocketAddr: Debug,
    SS: Displayable + Clone + Send + 'static,
    S: ServableStream<SS>,
{
    tracing::debug!("start serving connections");
    let error_message = format!("failed to bind to {}", listener_bind_to.display());
    let listener = L::bind(listener_bind_to).await.context(error_message)?;

    loop {
        let (stream, addr) = listener
            .accept()
            .await
            .context("failed to accept new connection")?;
        let (new_client_read_half, new_client_write_half) = stream.into_split();
        tracing::info!(?addr, "new connection from {new_client_name} incoming");

        tracing::debug!("creating new connection to {stream_name}");
        let error_message = format!("failed to connect to {}", stream_connect_to.display());
        let stream = match S::connect(stream_connect_to.clone()).await {
            Ok(stream) => stream,
            Err(error) => {
                if let Some(mut hook) = hooks.on_stream_connect_fail.take() {
                    hook(&error)
                }

                Err(error).context(error_message)?
            }
        };

        let (stream_read_half, stream_write_half) = stream.into_split();
        let (nc2s_finished_sender, mut nc2s_finished_receiver) = mpsc::channel(1);
        let (s2nc_finished_sender, mut s2nc_finished_receiver) = mpsc::channel(1);

        tracing::debug!("created new connection to {stream_name}");

        // read from new client, then send to stream
        tokio::spawn(async move {
            read_then_send_worker(
                &new_client_read_half,
                &stream_write_half,
                new_client_name,
                stream_name,
                nc2s_finished_sender,
            ).await
        });

        // read from stream, then send to new client
        tokio::spawn(async move {
            read_then_send_worker(
                &stream_read_half,
                &new_client_write_half,
                stream_name,
                new_client_name,
                s2nc_finished_sender,
            ).await
        });

        tokio::spawn(async move {
            let mut nc2s_finished = false;
            let mut s2nc_finished = false;

            loop {
                tokio::select! {
                    Some(()) = nc2s_finished_receiver.recv() => nc2s_finished = true,
                    Some(()) = s2nc_finished_receiver.recv() => s2nc_finished = true,
                }

                if nc2s_finished && s2nc_finished {
                    tracing::info!("connection to {stream_name} closed");
                    break
                }
            }
        });
    }
}
