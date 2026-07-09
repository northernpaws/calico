use std::{path::PathBuf, process::exit};

use bmputil::{BmpParams, bmp::BmpMatcher, serial::interface::ProbeInterface};
use calico::elf::TestElfInfo;
use calico_runner::probe::{DebugProbe, Probe};
use clap::Args;
use color_eyre::Result;
use std::str::FromStr;

use crate::gdb::GDBInterface;

#[derive(Args)]
pub(crate) struct RunArgs {
    #[clap(index = 1, help = "The path to the ELF file to flash and run.")]
    pub(crate) path: PathBuf,

    /// Use the device with the given serial number
    #[arg(global = true, short = 's', long = "serial", alias = "serial-number")]
    serial_number: Option<String>,

    /// Use the nth found device (may be unstable!)
    #[arg(global = true, long = "index", value_parser = usize::from_str)]
    index: Option<usize>,
}

impl BmpParams for RunArgs {
    fn index(&self) -> Option<usize> {
        self.index
    }

    fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }
}

/// Execute the run command.
pub(crate) async fn execute(args: &RunArgs) -> Result<()> {
    tracing::info!("starting calico runner {}", args.path.to_string_lossy());

    println!("starting calico runner: {}", args.path.to_string_lossy());

    // ELFs built for Calcio contain a .info section called `calico` that
    // contains test metadata used by the runner, but it discarded when
    // flashed the the target with a debug probe.
    //s
    // This allows metadata to accompany the ELF without needed to embed
    // the entire test metadata into the flashed binary, avoiding using
    // precious bytes on the target's flash memory.
    tracing::info!("reading ELF {}", args.path.to_string_lossy());
    let elf_info = TestElfInfo::from_elf(&args.path)
        .expect("failed to read test data from ELF .calico section");

    tracing::info!("got ELF info {:#?}", elf_info);

    // ====================
    // Flash the binary to the target.

    tracing::info!("Scanning for probe...");
    let probe: Probe = Probe::new_probe_rs();
    if !probe.has_probe() {
        tracing::error!("No probe found!");
    } else {
        tracing::info!("Probe found!");
    }

    tracing::info!("Flashing...");
    probe.flash(&args.path);
    tracing::info!("Flashing complete!");

    // Try and identify all the probes on the system that are allowed by the invocation
    /*let matcher = BmpMatcher::from_params(args);
    let mut results = matcher.find_matching_probes();
    let device: bmputil::bmp::BmpDevice = results.pop_single("run").map_err(|_| exit(1))?;

    tracing::info!("Probe identity: {}", device.firmware_identity()?);

    // Attempt to get a BMD serial command interface.
    // let remote = device.bmd_serial_interface()?.remote()?;

    // let remote_gdb = device.gdb_serial_interface()?;
    let serial_interface = ProbeInterface::from_device(&device)?;
    let gdb_interface_path = serial_interface.probe_gdb_interface()?;
    tracing::info!(
        "connecting to GDB on {}",
        gdb_interface_path.to_string_lossy()
    );

    // Open a handle to the GDB command line in machine interface mode.
    let mut gdb = GDBInterface::open();

    tracing::info!(
        "connecting to GDB remote target {}",
        gdb_interface_path.to_string_lossy()
    );

    // Connect the GDB client to the remote target.
    gdb.target_extended_remote(&gdb_interface_path.to_string_lossy().to_string())
        .expect("connection to remote target");

    tracing::info!("enabling probe tpwr");

    // Enable tpwr for nucleo boards.
    gdb.monitor(Some(vec!["tpwr".to_string(), "enable".to_string()]))
        .expect("successful tpwr enable");

    tracing::info!("running swd_scan");

    // Scan for connected targets.
    gdb.monitor(Some(vec!["swd_scan".to_string()]))
        .expect("successful swd_scan");*/

    Ok(())
}
