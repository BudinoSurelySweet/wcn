use crate::build_system::VersionControlSystem;
use clap::{
    Args, Parser, Subcommand,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};

const MENU_STYLE: Styles = Styles::styled()
    // Section headers (e.g., "Usage:", "Options:", "Commands:")
    .header(AnsiColor::White.on_default().effects(Effects::BOLD))
    // The actual word "Usage" at the beginning
    .usage(AnsiColor::BrightGreen.on_default().effects(Effects::BOLD))
    // Flags and literal commands (e.g., -h, --help, build)
    .literal(AnsiColor::White.on_default().effects(Effects::BOLD))
    // Values or arguments to substitute (e.g., <PROFILE>, <FILE>)
    .placeholder(AnsiColor::Yellow.on_default())
    // Descriptive text for invalid commands (if the user makes a mistake)
    .invalid(AnsiColor::BrightRed.on_default().effects(Effects::BOLD))
    // Descriptive text for valid commands
    .valid(AnsiColor::Green.on_default())
    // Error messages output by the CLI
    .error(AnsiColor::BrightRed.on_default().effects(Effects::BOLD));

#[derive(Parser, Debug)]
#[command(author = "Marco Molossi", version, about, styles = MENU_STYLE)]
pub struct ClapArgs {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Create a new project
    New(NewProjectCommand),

    /// Build the project
    Build(BuildCommand),

    /// Build the project and run it
    Run(RunCommand),

    Init,

    /// Clean the folder with outputs
    Clean,
}

#[derive(Args, Debug)]
pub struct NewProjectCommand {
    /// Name of the new project
    pub name: String,

    /// Override the default version control system
    #[clap(long, default_value = "git")]
    pub vcs: VersionControlSystem,
}

#[derive(Args, Debug)]
pub struct BuildCommand {
    /// The profile with which the project will be built. If empty the default will be used. You can
    /// set a default profile in the configuration file.
    pub profile: Option<String>,
}

#[derive(Args, Debug)]
pub struct RunCommand {
    /// The profile with which the project will be built. If empty the default will be used. You can
    /// set a default profile in the configuration file.
    pub profile: Option<String>,

    /// Arguments passed to the executable
    #[clap(last = true)]
    pub args: Vec<String>,
}
