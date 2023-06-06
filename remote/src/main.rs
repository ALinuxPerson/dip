use anyhow::Context;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::net::{TcpListener, UnixStream};
use dip_common::config::ConfigLike;
use dip_common::DEFAULT_PORT;
use dip_common::serve::ServeHooks;

#[derive(Serialize, Deserialize, Parser)]
#[command(author, version, about)]
pub struct Config {
    /// The port to accept host connections to. If not specified, it will default to 49131.
    #[clap(short = 'P', long)]
    pub port: Option<u16>,

    /// The location of the Discord IPC path. If not specified, it will be automatically detected.
    #[clap(short = 'p', long)]
    pub discord_ipc_path: Option<PathBuf>,
}

impl<'de> ConfigLike<'de> for Config {
    const FILE_NAME: &'static str = "remote.toml";
}

pub fn find_existing_socket() -> Option<PathBuf> {
    dip_common::find_socket(|socket_path| socket_path.exists())
}

fn fetch_local_ip(fetch_fn: impl FnOnce() -> Result<IpAddr, local_ip_address::Error>, port: u16, ip_type: &str) {
    match fetch_fn() {
        Ok(ip_address) => tracing::info!("remote {ip_type} address is {ip_address}:{port}"),
        Err(error) => tracing::warn!("failed to retrieve local {ip_type} address: {error}"),
    }
}

async fn try_main() -> anyhow::Result<()> {
    let (span, config) = dip_common::common::<Config>()?;
    let socket_path = config
        .discord_ipc_path
        .or_else(find_existing_socket)
        .context("no existing sockets are available (is discord open?)")?;
    tracing::info!("socket path is {}", socket_path.display());

    let port = config.port.unwrap_or(DEFAULT_PORT);
    tracing::info!(?port, "port to listen on");

    tracing::info!("successfully resolved configuration");
    drop(span);

    fetch_local_ip(local_ip_address::local_ip, port, "ipv4");
    fetch_local_ip(local_ip_address::local_ipv6, port, "ipv6");

    dip_common::serve::<TcpListener, UnixStream, _, _>(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
        socket_path,
        "host server",
        "discord ipc",
        ServeHooks::default()
            .on_stream_connect_fail(|_| tracing::warn!("was discord open then closed?")),
    ).await
}

#[tokio::main]
async fn main() -> ExitCode {
    dip_common::utils::try_main(try_main).await
}
