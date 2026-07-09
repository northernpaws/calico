#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;

// Bring DefaultHandler into scope.
//
// Required when using the link.x script that is provided by Embassy.
use embassy_stm32 as _;

// #[cortex_m_rt::entry]
// fn main() -> ! {
//     loop {}
// }

///
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
