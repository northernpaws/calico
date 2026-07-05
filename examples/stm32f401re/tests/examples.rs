#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

///
#[cfg(test)] // configures out the module outside of test runs
#[calico::tests]
mod example_test_module {
    #[init]
    fn init() {}

    #[test]
    fn test1() {}
}
