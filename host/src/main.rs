mod utils;

use crate::utils::MaybeSocketAddr;
use anyhow::Context;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::OnceLock;
use tokio::net::{TcpStream, UnixListener};
use dip_common::config::ConfigLike;
use dip_common::DEFAULT_PORT;
use dip_common::serve::ServeHooks;

#[derive(Serialize, Deserialize, Parser)]
#[command(author, version, about)]
pub struct Config {
    /// The location of the Discord IPC path. If not specified, it will be automatically detected.
    #[clap(short = 'p', long)]
    pub discord_ipc_path: Option<PathBuf>,

    /// The remote address that the host will connect and forward packets to.
    #[clap(short, long)]
    pub remote_address: Option<MaybeSocketAddr>,

    /// Whether or not to keep the unix socket created by this program on exit.
    #[clap(short, long)]
    #[serde(default)]
    pub keep_socket: bool,
}

impl<'de> ConfigLike<'de> for Config {
    const FILE_NAME: &'static str = "host.toml";
}

pub fn find_available_socket() -> Option<PathBuf> {
    dip_common::find_socket(|socket_path| !socket_path.exists())
}

async fn try_main() -> anyhow::Result<()> {
    let (span, config) = dip_common::common::<Config>()?;
    let socket_path = config
        .discord_ipc_path
        .or_else(find_available_socket)
        .context("no more available sockets available (too many discord clients open?)")?;
    tracing::info!("socket path is {}", socket_path.display());

    let remote_address = config
        .remote_address
        .with_context(|| {
            format!(
                "the remote address must be passed in either the arguments or the config ('{}')",
                Config::toml().display()
            )
        })?
        .with_port(DEFAULT_PORT);
    tracing::info!(%remote_address, "remote address to connect to");

    tracing::info!("successfully resolved configuration");
    drop(span);

    let destroy_socket_on_drop = OnceLock::new();

    if !config.keep_socket {
        let (destroy_on_drop, fut) = utils::destroy_path_on_termination(socket_path.clone())?;

        // we need to do this because we need `destroy_on_drop` to stay alive until the end of the
        // main function, otherwise it'll get dropped by the end of this scope and then the unix
        // socket gets deleted immediately, which is bad if we want external programs to interact
        // with our unix socket, cuz, y'know, it doesn't exist.
        destroy_socket_on_drop.set(destroy_on_drop)
            .unwrap_or_else(|_| panic!("`destroy_socket_on_drop` already set"));

        tokio::spawn(fut);
    }

    #[cfg(unix)]
    let new_client_name = "unix socket";

    #[cfg(windows)]
    let new_client_name = "named pipe";

    dip_common::serve::<UnixListener, TcpStream, _, _>(
        socket_path,
        remote_address,
        new_client_name,
        "remote client",
        ServeHooks::default()
            .on_stream_connect_fail(|_| tracing::warn!("is the remote client currently on right now?")),
    )
    .await
}

#[tokio::main]
async fn main() -> ExitCode {
    dip_common::utils::try_main(try_main).await
}
