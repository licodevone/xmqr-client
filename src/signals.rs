//! Process stop signals shared by network and input cancellation.
use std::io;

pub async fn shutdown() -> io::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        }
    }
    #[cfg(all(windows, feature = "windows-service-runtime"))]
    {
        crate::windows_service::shutdown().await
    }
    #[cfg(not(any(unix, all(windows, feature = "windows-service-runtime"))))]
    tokio::signal::ctrl_c().await
}
