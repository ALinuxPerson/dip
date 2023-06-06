use anyhow::Context;
use directories::ProjectDirs;
use std::sync::OnceLock;

static DIRS: OnceLock<ProjectDirs> = OnceLock::new();

pub fn initialize() -> anyhow::Result<()> {
    #[cfg(any(not(windows), not(target_os = "macos")))]
    let application = "dip";

    #[cfg(any(windows, target_os = "macos"))]
    let application = "DIP";

    DIRS.set(
        ProjectDirs::from("", "ALinuxPerson", application)
            .context("could not find project directories")?,
    )
    .unwrap_or_else(|_| panic!("`DIRS` already initialized"));

    Ok(())
}

pub fn dirs() -> &'static ProjectDirs {
    DIRS.get().unwrap()
}
