use anyhow::Result;
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logger() -> Result<()> {
    // Create a console layer for stdout
    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact();

    // Create an environment filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("dos_safar=info,info"));

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(console_layer)
        .with(filter)
        .init();

    Ok(())
}

// Helper macros for colored output in gaming mode
#[macro_export]
macro_rules! gaming_info {
    ($($arg:tt)*) => {
        tracing::info!("🎮 {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! gaming_warn {
    ($($arg:tt)*) => {
        tracing::warn!("⚠️  {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! gaming_error {
    ($($arg:tt)*) => {
        tracing::error!("❌ {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! gaming_success {
    ($($arg:tt)*) => {
        tracing::info!("✅ {}", format!($($arg)*));
    };
}