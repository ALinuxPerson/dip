use anyhow::Context;
use fs_err::tokio as fs;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;
use tokio::signal::unix;
use tokio::signal::unix::SignalKind;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;

fn multiple_signals(
    signals: impl IntoIterator<Item = SignalKind>,
) -> anyhow::Result<UnboundedReceiver<()>> {
    let (sender, receiver) = mpsc::unbounded_channel();
    let signals = signals
        .into_iter()
        .map(unix::signal)
        .collect::<Result<Vec<_>, _>>()
        .context("failed to register all signals")?;

    for mut signal in signals {
        let sender = sender.clone();

        tokio::spawn(async move {
            loop {
                if signal.recv().await.is_some() {
                    sender.send(()).unwrap();
                } else {
                    break;
                }
            }
        });
    }

    Ok(receiver)
}

async fn destroy_path(path: impl AsRef<Path>) {
    let path = path.as_ref();

    if let Err(error) = fs::remove_file(path).await {
        tracing::error!(?error, "failed to remove unix socket")
    } else {
        tracing::debug!(?path, "removed unix socket")
    }
}

pub struct DestroyPathOnDrop(pub PathBuf);

impl Drop for DestroyPathOnDrop {
    fn drop(&mut self) {
        tokio::spawn(destroy_path(self.0.clone()));
    }
}

pub fn destroy_path_on_termination(
    path: PathBuf,
) -> anyhow::Result<(DestroyPathOnDrop, impl Future<Output = ()>)> {
    let mut signals = multiple_signals([
        SignalKind::terminate(),
        SignalKind::interrupt(),
        SignalKind::quit(),
    ])
    .context("failed to register SIGTERM, SIGINT, and SIGQUIT signals")?;

    tracing::debug!("destroy path {} on termination", path.display());

    let destroy_path_on_drop = DestroyPathOnDrop(path.clone());

    Ok((destroy_path_on_drop, async move {
        signals.recv().await;
        destroy_path(path).await;
        process::exit(0)
    }))
}

#[derive(Clone, Copy)]
pub struct MaybeSocketAddr {
    pub address: IpAddr,
    pub port: Option<u16>,
}

impl MaybeSocketAddr {
    pub fn to_socket_addr(self) -> Option<SocketAddr> {
        self.port.map(|port| SocketAddr::new(self.address, port))
    }

    pub fn with_port(self, port: u16) -> SocketAddr {
        SocketAddr::new(self.address, self.port.unwrap_or(port))
    }
}

impl Serialize for MaybeSocketAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(socket_addr) = self.to_socket_addr() {
            socket_addr.serialize(serializer)
        } else {
            self.address.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for MaybeSocketAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::from_str(&String::deserialize(deserializer)?).map_err(|e| {
            D::Error::custom(format!(
                "socket address or ip address could not be deserialized successfully: {e}"
            ))
        })
    }
}

impl FromStr for MaybeSocketAddr {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SocketAddr::from_str(s).map(Self::from).or_else(|_| {
            Ok(Self::from(
                IpAddr::from_str(s).context("failed to parse ip address")?,
            ))
        })
    }
}

impl From<SocketAddr> for MaybeSocketAddr {
    fn from(value: SocketAddr) -> Self {
        Self {
            address: value.ip(),
            port: Some(value.port()),
        }
    }
}

impl From<IpAddr> for MaybeSocketAddr {
    fn from(value: IpAddr) -> Self {
        Self {
            address: value,
            port: None,
        }
    }
}
