use std::sync::OnceLock;
use anyhow::Context;
use directories::ProjectDirs;

static DIRS: OnceLock<ProjectDirs> = OnceLock::new();

pub fn initialize() -> anyhow::Result<()> {
    DIRS.set(ProjectDirs::from("", "ALinuxPerson", "dip")
        .context("could not find project directories")?)
        .unwrap_or_else(|_| panic!("`DIRS` already initialized"));

    Ok(())
}

pub fn dirs() -> &'static ProjectDirs {
    DIRS.get().unwrap()
}
