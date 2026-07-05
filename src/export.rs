/// Embed the version of Calico that the binary was compiled with.
///
/// This defines the Calico protocol version exposed
/// by the compiled binary to the Calico test runner.
#[cfg(not(target_os = "macos"))] // fixes section name format error on Mac
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".calico")]
static CALICO_VERSION: usize = 0;
