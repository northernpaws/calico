/// Embed the version of Calico that the binary was compiled with.
///
/// This defines the Calico protocol version exposed
/// by the compiled binary to the Calico test runner.
#[cfg(not(target_os = "macos"))] // fixes section name format error on Mac
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".calico")]
static CALICO_VERSION: usize = 0;

// Ariel OS invokes the `__calico_entry` function directly
// Otherwise we export it as `main` function.
#[unsafe(export_name = "main")]
pub unsafe extern "C" fn __calico_entry() -> ! {
    ensure_linker_file_was_added_to_rustflags();
}

fn ensure_linker_file_was_added_to_rustflags() -> ! {
    // Try to access a symbol which we provide in the calico.x linker script.
    //
    // The linker script will redirect this call to the function below.
    //
    // This will trigger a linker error if the linker script has not been added to the rustflags
    unsafe extern "C" {
        fn calico_linker_file_not_added_to_rustflags() -> !;
    }

    unsafe { calico_linker_file_not_added_to_rustflags() }
}

/// The entrypoint for the test harness.
#[unsafe(no_mangle)]
unsafe extern "C" fn __calico_start() -> ! {
    // Invoke the user provided setup function if it
    // exists, or run a default (empty) setup function.
    unsafe extern "Rust" {
        fn _calico_setup();
    }

    unsafe { _calico_setup() }

    // TODO: setup and run interactions with host/runner

    loop {}
}

/// The default sartup function called if no user-defined startup function has been provided.
#[unsafe(export_name = "__calico_default_setup")]
fn default_setup() {}
