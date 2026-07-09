//! This module provides the traits and implementations
//! for interfacing with debugging probes.

use std::path::PathBuf;

#[cfg(feature = "probe-rs")]
use crate::probe::probe_rs::ProbeRs;

#[cfg(feature = "probe-rs")]
pub mod probe_rs;

/// Top-level definition of all possible probe
/// types for flashing and debugging.
pub enum Probe {
    /// Uses the probe-rs library to interface
    /// with a variety of probes.
    #[cfg(feature = "probe-rs")]
    ProbeRs(ProbeRs),
}

impl Probe {
    pub fn new_probe_rs() -> Self {
        Self::ProbeRs(ProbeRs {})
    }
}

/// Defines the base trait used to implement debug probes.
pub trait DebugProbe {
    fn has_probe(&self) -> bool;
    fn flash(&self, binary_path: &PathBuf);
}

impl DebugProbe for Probe {
    fn has_probe(&self) -> bool {
        match self {
            Probe::ProbeRs(probe_rs) => probe_rs.has_probe(),
        }
    }

    fn flash(&self, binary_path: &PathBuf) {
        match self {
            Probe::ProbeRs(probe_rs) => probe_rs.flash(binary_path),
        }
    }
}
