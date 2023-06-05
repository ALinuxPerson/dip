use std::fmt::{Debug, Display};
use std::{io, thread};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use anyhow::Context;
use async_trait::async_trait;
use tokio::net::{tcp, TcpListener, TcpStream, ToSocketAddrs, unix, UnixListener, UnixStream};
use tokio::net::unix::SocketAddr as UnixSocketAddr;
use std::path::Display as DisplayablePath;
use std::time::Duration;
use crate::{ReadFrom, WriteTo};

#[async_trait]
pub trait ServableListener<S: Send + 'static>: Sized {
    type Stream: ServableStream<S>;
    type SocketAddr;

    async fn bind(socket: S) -> io::Result<Self>;
    async fn accept(&self) -> io::Result<(Self::Stream, Self::SocketAddr)>;
}

#[async_trait]
impl<S: AsRef<Path> + Send + 'static> ServableListener<S> for UnixListener {
    type Stream = UnixStream;
    type SocketAddr = UnixSocketAddr;

    async fn bind(socket: S) -> io::Result<Self> {
        UnixListener::bind(socket)
    }

    async fn accept(&self) -> io::Result<(Self::Stream, Self::SocketAddr)> {
        UnixListener::accept(self).await
    }
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
    type ReadHalf<'a>: ReadFrom + Send where Self: 'a;
    type WriteHalf<'a>: WriteTo + Send where Self: 'a;

    async fn connect(socket: S) -> io::Result<Self>;
    fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf);
    fn split<'a>(&'a mut self) -> (Self::ReadHalf<'a>, Self::WriteHalf<'a>);
}

#[async_trait]
impl<S: AsRef<Path> + Send + 'static> ServableStream<S> for UnixStream {
    type OwnedReadHalf = unix::OwnedReadHalf;
    type OwnedWriteHalf = unix::OwnedWriteHalf;
    type ReadHalf<'a> = unix::ReadHalf<'a> where Self: 'a;
    type WriteHalf<'a> = unix::WriteHalf<'a> where Self: 'a;

    async fn connect(socket: S) -> io::Result<Self> {
        UnixStream::connect(socket).await
    }

    fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
        UnixStream::into_split(self)
    }

    fn split<'a>(&'a mut self) -> (Self::ReadHalf<'a>, Self::WriteHalf<'a>) {
        UnixStream::split(self)
    }
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

    fn split<'a>(&'a mut self) -> (Self::ReadHalf<'a>, Self::WriteHalf<'a>) {
        TcpStream::split(self)
    }
}

pub trait Displayable {
    type Display<'d>: Display where Self: 'd;

    fn display<'d>(&'d self) -> Self::Display<'d>;
}

impl Displayable for PathBuf {
    type Display<'d> = DisplayablePath<'d> where Self: 'd;

    fn display<'d>(&'d self) -> Self::Display<'d> {
        Path::display(self)
    }
}

macro_rules! impl_Displayable {
    ($ty:ty) => {
        impl Displayable for $ty {
            type Display<'d> = &'d $ty;

            fn display<'d>(&'d self) -> Self::Display<'d> {
                self
            }
        }
    };
}

impl_Displayable!(SocketAddr);

pub async fn serve<L, S, LS, SS>(listener_bind_to: LS, stream_connect_to: SS, new_client_name: &'static str, stream_name: &'static str) -> anyhow::Result<()>
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
        let (stream, addr) = listener.accept().await.context("failed to accept new connection")?;
        let (new_client_read_half, new_client_write_half) = stream.into_split();
        tracing::info!(?addr, "new connection from {new_client_name} incoming");

        tracing::debug!("creating new connection to {stream_name}");
        let error_message = format!("failed to connect to {}", stream_connect_to.display());
        let mut stream = S::connect(stream_connect_to.clone()).await.context(error_message)?;

        let (stream_read_half, stream_write_half) = stream.into_split();

        tracing::debug!("created new connection to {stream_name}");

        // read from new client, then send to stream
        tokio::spawn(async move {
            tracing::trace!("new client->stream worker started");

            loop {
                if crate::read_from_then_write_to(&new_client_read_half, &stream_write_half, new_client_name, stream_name).await.is_break() {
                    tracing::debug!("finished reading from {new_client_name} and sending to {stream_name}");
                    break
                }
            }
        });

        // read from stream, then send to new client
        tokio::spawn(async move {
            tracing::trace!("stream->new client worker started");

            loop {
                if crate::read_from_then_write_to(&stream_read_half, &new_client_write_half, stream_name, new_client_name).await.is_break() {
                    tracing::debug!("finished reading from {stream_name} and sending to {new_client_name}");
                    break
                }
            }
        });
    }
}
