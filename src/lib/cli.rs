//! Command line interface.

use clap::Shell;
use structopt::StructOpt;

use ic_agent::Agent;
use ic_types::Principal;

/// Internet Computer Mini Terminal (ic-mt)
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "ic-mt", raw(setting = "clap::AppSettings::DeriveDisplayOrder"))]
pub struct CliOpt {
    /// Path for output files with event and screen captures.
    #[structopt(short = "o", long = "out", default_value = "./out")]
    pub capture_output_path: String,
    /// Frame rate (uniform) for producing captured GIF files with engiffen.
    #[structopt(long = "engiffen-framerate", default_value = "16")]
    pub engiffen_frame_rate: usize,
    /// Suppress window for graphics output.
    #[structopt(short = "W", long = "no-window")]
    pub no_window: bool,
    /// Suppress capturing graphics output.
    #[structopt(short = "C", long = "no-capture")]
    pub no_capture: bool,
    /// Trace-level logging (most verbose)
    #[structopt(short = "t", long = "trace-log")]
    pub log_trace: bool,
    /// Debug-level logging (medium verbose)
    #[structopt(short = "d", long = "debug-log")]
    pub log_debug: bool,
    /// Coarse logging information (not verbose)
    #[structopt(short = "L", long = "log")]
    pub log_info: bool,
    #[structopt(subcommand)]
    pub command: CliCommand,
}

#[derive(StructOpt, Debug, Clone)]
pub enum CliCommand {
    #[structopt(
        name = "completions",
        about = "Generate shell scripts for auto-completions."
    )]
    Completions { shell: Shell },
    #[structopt(name = "connect", about = "Connect to an IC canister.")]
    Connect {
        replica_url: String,
        canister_id: String,
        /// Initialization arguments, as a Candid textual value (default is empty tuple).
        #[structopt(short = "i", long = "user")]
        user_info_text: String,
    },
}

/// Connection context: IC agent object, for server calls, and configuration info.
pub struct ConnectCtx {
    pub cfg: ConnectCfg,
    pub agent: Agent,
    pub canister_id: Principal,
}

/// Connection configuration
#[derive(Debug, Clone)]
pub struct ConnectCfg {
    pub cli_opt: CliOpt,
    pub canister_id: String,
    pub replica_url: String,
    /// temp hack: username and user-chosen color
    pub user_info: crate::types::UserInfoCli,
}
