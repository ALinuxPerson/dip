use std::future::Future;
use std::process::ExitCode;

pub async fn try_main<F: Future<Output = anyhow::Result<()>>>(
    main_fn: impl FnOnce() -> F,
) -> ExitCode {
    if let Err(error) = main_fn().await {
        tracing::error!("{error:#}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
