use probe_rs::{
    Permissions, UnwindRule,
    config::TargetSelector,
    flashing::{
        DownloadOptions, FlashProgress, FormatKind, build_loader, download_file,
        download_file_with_options,
    },
    probe::{Probe, list::Lister},
};

/// Implements a probe interface using the probe-rs
/// library to support a wide range of debug probes.
pub struct ProbeRs {}

impl super::DebugProbe for ProbeRs {
    fn has_probe(&self) -> bool {
        let probe_lister = Lister::new();

        // Get a list of all probes, and use the first probe found.
        let probes = probe_lister.list_all();
        if probes.len() == 0 {
            return false;
        }

        return true;
    }

    fn flash(&self, binary_path: &std::path::PathBuf) {
        let probe_lister = Lister::new();

        // Get a list of all probes, and use the first probe found.
        let probes = probe_lister.list_all();
        let mut probe: Probe = probes[0].open().expect("failed to find probe");

        // Select SWD.
        probe
            .select_protocol(probe_rs::probe::WireProtocol::Swd)
            .expect("Failed to select SWD as the transport protocol");

        // Attempt to attach to a target connected to the probe.
        let target_selector = TargetSelector::Unspecified("stm32f401re".to_string());
        //  NOTE: there is also attach_to_unspecified
        let mut session = probe
            .attach(target_selector, Permissions::default())
            .expect("Failed to attach probe to target");

        println!("Target name: {:?}", session.target().name);
        println!("Target architecture: {:?}", session.target().architecture());
        println!("Target memory map: {:?}", session.target().memory_map);

        // Load the memory map.
        // let mm = session.memory_map();

        let progress = FlashProgress::new(|event| println!("Event: {:#?}", event));
        let mut options = DownloadOptions::default();
        options.progress = progress;
        options.skip_erase = false;
        options.do_chip_erase = true;
        options.preverify = true;

        // Download the binary to flash.
        // download_file_with_options(&mut session, binary_path, FormatKind::Elf, options).unwrap();

        let loader = build_loader(&mut session, binary_path, FormatKind::Elf.into(), None).unwrap();

        loader.commit(&mut session, options).unwrap()
    }
}
