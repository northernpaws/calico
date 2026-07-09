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

// Checks the outcome of a test.
pub fn check_outcome<T: TestOutcome>(outcome: T) -> ! {
    if outcome.is_success() {
        info!("Test exited with () or Ok(..)");
        // hosting::exit(0);
        loop {}
    } else {
        info!("Test exited with Err(..): {:?}", outcome);
        // hosting::abort();
        loop {}
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for () {}
    impl<T, E> Sealed for Result<T, E> {}
}

/// Indicates whether a test succeeded or failed.
///
/// This is comparable to the `Termination` trait in libstd, except stable and tailored towards the
/// needs of embedded-test. It is implemented for `()`, which always indicates success, and `Result`,
/// where `Ok` indicates success.
#[cfg(feature = "defmt")]
pub trait TestOutcome: defmt::Format + sealed::Sealed {
    fn is_success(&self) -> bool;
}

/// Indicates whether a test succeeded or failed.
///
/// This is comparable to the `Termination` trait in libstd, except stable and tailored towards the
/// needs of embedded-test. It is implemented for `()`, which always indicates success, and `Result`,
/// where `Ok` indicates success.
#[cfg(feature = "log")]
pub trait TestOutcome: core::fmt::Debug + sealed::Sealed {
    fn is_success(&self) -> bool;
}

/// Indicates whether a test succeeded or failed.
///
/// This is comparable to the `Termination` trait in libstd.
///
/// It is implemented for `()`, which always indicates success,
/// and `Result`, where `Ok` indicates success.
#[cfg(all(not(feature = "log"), not(feature = "defmt")))]
pub trait TestOutcome: sealed::Sealed {
    fn is_success(&self) -> bool;
}

impl TestOutcome for () {
    fn is_success(&self) -> bool {
        true
    }
}

#[cfg(feature = "log")]
impl<T: core::fmt::Debug, E: core::fmt::Debug> TestOutcome for Result<T, E> {
    fn is_success(&self) -> bool {
        self.is_ok()
    }
}

#[cfg(feature = "defmt")]
impl<T: defmt::Format, E: defmt::Format> TestOutcome for Result<T, E> {
    fn is_success(&self) -> bool {
        self.is_ok()
    }
}

#[cfg(all(not(feature = "log"), not(feature = "defmt")))]
impl<T, E> TestOutcome for Result<T, E> {
    fn is_success(&self) -> bool {
        self.is_ok()
    }
}
