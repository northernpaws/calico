//! Provides instrumentation specific to tests.

#[cfg(feature = "tracing")]
use tracing::{Level, event};

/// Marks the start of a test.
pub fn test_start() {
    // Log the start of the test, if logging is enabled.
    info!("starting test");

    // Emit a tracing event for the start of the
    // test, if tracing support is enabled.
    #[cfg(feature = "tracing")]
    event!(Level::INFO, "start");
}

/// Marks the end of a test.
pub fn test_end() {
    // Log the end of the test, if logging is enabled.
    info!("ending test");

    // Emit a tracing event for the end of the
    // test, if tracing support is enabled.
    #[cfg(feature = "tracing")]
    event!(Level::INFO, "end");
}
