use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

static REMOTE_TOML: Lazy<PathBuf> =
    Lazy::new(|| dip_common::dirs().config_dir().join("remote.toml"));

pub fn remote_toml() -> &'static Path {
    &REMOTE_TOML
}
