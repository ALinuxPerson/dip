mod utils;
mod dirs {
    use std::path::{Path, PathBuf};
    use once_cell::sync::Lazy;

    static HOST_TOML: Lazy<PathBuf> = Lazy::new(|| dip_common::dirs().config_dir().join("host.toml"));

    pub fn host_toml() -> &'static Path {
        &HOST_TOML
    }
}

use std::path::PathBuf;
use std::process::ExitCode;
use anyhow::Context;
use clap::Parser;
use figment::Figment;
use figment::providers::{Format, Serialized, Toml};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpStream, UnixListener};
use crate::utils::MaybeSocketAddr;

/// Host program for DIP.
#[derive(Serialize, Deserialize, Parser)]
pub struct Config {
    /// The location of the Discord IPC path. If not specified, it will automatically detect it.
    #[clap(short = 'p', long)]
    pub discord_ipc_path: Option<PathBuf>,

    /// The remote address that the server will connect and forward packets to.
    #[clap(short, long)]
    pub remote_address: Option<MaybeSocketAddr>,
}

impl Config {
    pub fn read() -> anyhow::Result<Self> {
        Figment::new()
            .merge(Serialized::defaults(Self::parse()))
            .merge(Toml::file(dirs::host_toml()))
            .extract()
            .context("failed to extract config")
    }
}

pub fn find_available_socket() -> Option<PathBuf> {
    dip_common::find_socket(|socket_path| !socket_path.exists())
}

async fn try_main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dip_common::dirs::initialize()?;

    let span = tracing::info_span!("resolve config");
    let config = Config::read()?;
    let socket_path = config.discord_ipc_path
        .or_else(find_available_socket)
        .context("no more available sockets available (too many discord clients open?)")?;
    tracing::info!("socket path is {}", socket_path.display());

    let remote_address = config.remote_address.with_context(|| format!("the remote address must be passed in either the arguments or the config ('{}')", dirs::host_toml().display()))?.with_port(49131);
    tracing::info!(%remote_address, "remote address to connect to");

    tracing::info!("successfully resolved configuration");
    drop(span);

    let (_destroy_on_drop, fut) = utils::destroy_path_on_termination(socket_path.clone())?;

    tokio::spawn(fut);

    dip_common::serve::<UnixListener, TcpStream, _, _>(
        socket_path,
        remote_address,
        "unix socket",
        "remote client",
    ).await
}

#[tokio::main]
async fn main() -> ExitCode {
    dip_common::utils::try_main(try_main).await
}