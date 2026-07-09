#![cfg_attr(not(feature = "std"), no_std)]

// This must go FIRST so that all the other modules see its macros.
mod log;

// If instrumentation is enabled, we re-export
// at the top level to avoid unnecessarily
// verbose paths for imports in tests.
#[cfg(all(feature = "harness", not(feature = "std")))]
mod harness;
#[cfg(all(feature = "harness", not(feature = "std")))]
pub use harness::*;

/// Shadows the macros crate to make paths cleaner.
#[cfg(feature = "macros")]
pub use calico_macros::*;

/// Enables features for reading/writing
/// ELF data related to the test harness.
#[cfg(feature = "elf")]
pub mod elf {
    pub use calico_elf::*;
}

/// Contains structures for encoding and
/// decoding metadata about tests.
#[cfg(feature = "rpc")]
pub mod rpc {
    pub use calico_rpc::*;
}
