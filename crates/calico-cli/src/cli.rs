use clap::builder::styling::Styles;

use clap::crate_version;
use color_eyre::config::HookBuilder;
use color_eyre::eyre::{Context, EyreHandler, InstallError, OptionExt, Result, eyre};
use owo_colors::OwoColorize;

/// Clap v3 style (approximate)
/// See https://stackoverflow.com/a/75343828
pub(crate) fn style() -> clap::builder::Styles {
    Styles::styled()
        .usage(
            anstyle::Style::new()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow)))
                .bold(),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
}

/// Installs a color_eyre error handler for custom formatting on panics.
pub(crate) fn install_error_handler() -> Result<()> {
    // Grab us a new default handler
    let default_handler = HookBuilder::default();
    // Turn that into a pair of hooks - one for panic, and the other for errors
    let (panic_hook, eyre_hook) = default_handler.try_into_hooks()?;

    // Make an instance of our custom handler, paassing it the panic one to do normal panic
    // handling with, so we only have to deal with our additions, and install it
    CalicoPanic {
        inner_hook: panic_hook.into_panic_hook(),
    }
    .install();

    // Make an instance of our custom handler, passing it the default one to do the main
    // error handling with, so we only have to deal with our additions, and install it
    CalicoHook {
        inner_hook: eyre_hook.into_eyre_hook(),
    }
    .install()?;
    Ok(())
}

type EyreHookFunc =
    Box<dyn Fn(&(dyn std::error::Error + 'static)) -> Box<dyn EyreHandler> + Send + Sync + 'static>;
type PanicHookFunc = Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Send + Sync + 'static>;

struct CalicoHook {
    inner_hook: EyreHookFunc,
}

struct CalicoPanic {
    inner_hook: PanicHookFunc,
}

struct CalicoHandler {
    inner_handler: Box<dyn EyreHandler>,
}

impl CalicoHook {
    fn build_handler(&self, error: &(dyn std::error::Error + 'static)) -> CalicoHandler {
        CalicoHandler {
            inner_handler: (*self.inner_hook)(error),
        }
    }

    pub fn install(self) -> Result<(), InstallError> {
        color_eyre::eyre::set_hook(self.into_eyre_hook())
    }

    pub fn into_eyre_hook(self) -> EyreHookFunc {
        Box::new(move |err| Box::new(self.build_handler(err)))
    }
}

impl CalicoPanic {
    pub fn install(self) {
        std::panic::set_hook(self.into_panic_hook());
    }

    pub fn into_panic_hook(self) -> PanicHookFunc {
        Box::new(move |panic_info| {
            self.print_header();
            (*self.inner_hook)(panic_info);
            self.print_footer();
        })
    }

    fn print_header(&self) {
        eprintln!("------------[ ✂ cut here ✂ ]------------");
        eprintln!(
            "Unhandled crash in calico-cli v{} ({})",
            crate_version!(),
            std::env::consts::OS
        );
        eprintln!();
    }

    fn print_footer(&self) {
        eprintln!();
        eprintln!(
            "{}",
            "Please include all lines down to this one from the cut here".yellow()
        );
        eprintln!(
            "{}",
            "marker, and report this issue to our issue tracker at".yellow()
        );
        eprintln!("https://github.com/northernpaws/calico/issues");
    }
}

impl EyreHandler for CalicoHandler {
    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        writeln!(fmt, "------------[ ✂ cut here ✂ ]------------")?;
        write!(fmt, "Unhandled crash in calico-cli v{}", crate_version!())?;
        self.inner_handler.debug(error, fmt)?;
        writeln!(fmt)?;
        writeln!(fmt)?;
        writeln!(
            fmt,
            "{}",
            "Please include all lines down to this one from the cut here".yellow()
        )?;
        writeln!(
            fmt,
            "{}",
            " marker, and report this issue to our issue tracker at".yellow()
        )?;
        write!(fmt, "https://github.com/northernpaws/calico/issues")
    }

    fn track_caller(&mut self, location: &'static std::panic::Location<'static>) {
        self.inner_handler.track_caller(location);
    }
}
