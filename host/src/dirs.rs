use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

static HOST_TOML: Lazy<PathBuf> =
    Lazy::new(|| dip_common::dirs().config_dir().join("host.toml"));

pub fn host_toml() -> &'static Path {
    &HOST_TOML
}
