mod dirs;

use anyhow::Context;
use clap::Parser;
use figment::providers::{Format, Serialized, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::net::{TcpListener, UnixStream};

/// Remote program for DIP.
#[derive(Serialize, Deserialize, Parser)]
pub struct Config {
    /// The port to accept incoming connections to.
    #[clap(short = 'P', long)]
    pub port: Option<u16>,

    /// The location of the Discord IPC path. If not specified, it will automatically detect it.
    #[clap(short = 'p', long)]
    pub discord_ipc_path: Option<PathBuf>,
}

impl Config {
    pub fn read() -> anyhow::Result<Self> {
        Figment::new()
            .merge(Serialized::defaults(Self::parse()))
            .merge(Toml::file(dirs::remote_toml()))
            .extract()
            .context("failed to extract config")
    }
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
    tracing_subscriber::fmt::init();
    dip_common::dirs::initialize()?;

    let span = tracing::info_span!("resolve config");
    let config = Config::read()?;

    let socket_path = config
        .discord_ipc_path
        .or_else(find_existing_socket)
        .context("no existing sockets are available (is discord open?)")?;
    tracing::info!("socket path is {}", socket_path.display());

    let port = config.port.unwrap_or(49131);
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
    )
    .await
}

#[tokio::main]
async fn main() -> ExitCode {
    dip_common::utils::try_main(try_main).await
}
