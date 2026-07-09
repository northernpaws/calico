#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("==============================================");
    info!("Calico HITL STM32F401RE Embassy Test");
    info!("==============================================");

    // // Initialize the HITL framework.
    // calico::init();

    // // Waits for a signal from the host, and
    // // then starts running the test suites.
    // calico::start().await;

    // test_1();
}

// /// This is a single functional test.
// #[calico::test(name = "test1234")]
// fn test_1() {}

// /// This is an example of a test suite.
// #[calico::tests]
// mod example_tests {
//     #[init]
//     fn init() {}

//     #[test]
//     fn test1() {}

//     #[test]
//     fn test2() {}

//     #[test]
//     fn test3() {}
// }
