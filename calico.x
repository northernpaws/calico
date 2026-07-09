# This linker script allows Calico to inject custom test
# info into the ELF file via Rust's link-section macros.

# The EMBEDDED_TEST_VERSION symbol is needed by probe-rs to determine whether a binary contains embedded tests or not
# Afterwards it reads the testcases from the .embedded_test section

# Redirect/rename a function here, so that we can make sure
# the user has added the linker script to the RUSTFLAGS.

EXTERN (__calico_start);
PROVIDE(calico_linker_file_not_added_to_rustflags = __calico_start);

PROVIDE(_calico_setup = __calico_default_setup);

# Define a new INFO section that holds to test metadata, and provides
# the test symbols from being optimized out by the linker.
SECTIONS
{
  .calico 1 (INFO) :
  {
    KEEP(*(.calico.*));
  }
}

# NOTE: build.rs will add a `INSERT AFTER .comment;` here, if we're compiling for std