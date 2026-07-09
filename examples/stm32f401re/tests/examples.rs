#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;

// Bring DefaultHandler and other
// Embassy linker symbols into scope.
//
// Required when using the link.x script
// that is provided by Embassy.
use embassy_stm32 as _;

/// A setup function called to configure the
/// test harness for running the test suites.
#[calico::setup]
fn setup() {
    // TODO: set up serial link or RTT
}

#[cfg(test)] // configures out the module outside of test runs
#[calico::tests]
mod example_test_module {
    #[init]
    fn init() {
        defmt::info!("Running init!");
    }

    #[test]
    fn test1() {
        defmt::info!("Running test1!");
    }

    #[test]
    fn test2() {
        defmt::info!("Running test2!");
    }
}
