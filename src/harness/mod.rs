//! Provides the methods for initializing
//! and instrumenting Calico tests.

/// Provides test-specific instrumentation.
pub mod test;

/// Emits `tracing` crate spans and events from testpoints.
#[cfg(feature = "tracing")]
pub mod tracing;

/// Called from a test to initialize the framework.
pub fn init() {}

/// Waits for a signal from the host, and then starts running the test suites.
///
/// Async in case background routines for things like MCU peripherals need to
/// run while the target waits for the signal from the test runner to start.
pub async fn start() {
    // let my_subscriber = Trac::new();
    // tracing::subscriber::with_default(my_subscriber, || {
    //     // Any trace events generated in this closure or by functions it calls
    //     // will be collected by `my_subscriber`.
    // })

    // If tracing support is enabled, we need to register the instrumentation
    // subscriber on the target to forward tracing events to the host.
    #[cfg(feature = "tracing")]
    let my_subscriber = tracing::TracingSubscriber::new();
    #[cfg(feature = "tracing")]
    ::tracing::subscriber::set_global_default(my_subscriber)
        .expect("setting tracing default failed");
}
