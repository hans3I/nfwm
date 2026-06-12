use clap::{Parser, Subcommand};
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "nfwm")]
#[command(about = "A Windows tiling window manager")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in shadow mode (read-only, no window movement)
    Shadow,
    /// Send an action to a running instance
    Action {
        /// The action name to send
        name: String,
    },
}

fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    info!("nfwm v{} starting", env!("CARGO_PKG_VERSION"));

    match cli.command {
        Some(Commands::Shadow) => {
            info!("Running in shadow mode");
            run_shadow_mode();
        }
        Some(Commands::Action { name }) => {
            info!("Sending action: {}", name);
            send_action(&name);
        }
        None => {
            info!("Running in normal mode");
            run_normal();
        }
    }
}

fn run_shadow_mode() {
    info!("Shadow mode: discovering windows without moving them");
    // TODO: Implement window discovery in Phase 03
    info!("Shadow mode complete. No windows were moved.");
}

fn run_normal() {
    info!("Normal mode: starting tiling manager");
    // TODO: Implement full tiling in later phases
    info!("Normal mode startup complete.");
}

fn send_action(name: &str) {
    info!("Attempting to send action '{}' to running instance", name);
    // TODO: Implement IPC in Phase 09
    warn!("IPC not yet implemented. Action '{}' not sent.", name);
}
