use clap::{Parser, Subcommand};
use nfwm_core::traits::{DisplayProvider, WindowProvider};
use nfwm_win32::display::Win32DisplayManager;
use nfwm_win32::window::Win32WindowManager;
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
    /// Diagnostic: enumerate windows, displays, and test Win32 APIs
    Diagnose,
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
        Some(Commands::Diagnose) => {
            info!("Running diagnostics");
            run_diagnostics();
        }
        None => {
            info!("Running in normal mode");
            run_normal();
        }
    }
}

fn run_shadow_mode() {
    info!("Shadow mode: discovering windows without moving them");
    let wm = Win32WindowManager::new();
    let windows = wm.enumerate_windows();
    info!("Found {} candidate windows", windows.len());
    for id in windows {
        info!("{}", wm.describe_window(id));
    }
    info!("Shadow mode complete. No windows were moved.");
}

fn run_normal() {
    info!("Normal mode: starting tiling manager");
    info!("Normal mode startup complete.");
}

fn send_action(name: &str) {
    info!("Attempting to send action '{}' to running instance", name);
    warn!("IPC not yet implemented. Action '{}' not sent.", name);
}

fn run_diagnostics() {
    info!("=== nfwm Diagnostics ===");

    // 1. Enumerate windows
    info!("--- Windows ---");
    let wm = Win32WindowManager::new();
    let windows = wm.enumerate_windows();
    info!("Found {} top-level windows", windows.len());
    for (i, id) in windows.iter().take(10).enumerate() {
        info!("  {}: {}", i + 1, wm.describe_window(*id));
    }
    if windows.len() > 10 {
        info!("  ... and {} more", windows.len() - 10);
    }

    // 2. Enumerate displays
    info!("--- Displays ---");
    let dm = Win32DisplayManager::new();
    let displays = dm.enumerate_monitors();
    info!("Found {} display(s)", displays.len());
    for (i, monitor) in displays.iter().enumerate() {
        let primary = if monitor.is_primary { " [PRIMARY]" } else { "" };
        let dpi = dm
            .dpi(monitor.id)
            .map(|d| format!(" DPI: {}", d))
            .unwrap_or_default();
        info!(
            "  {}: {}x{} at ({},{}) work: {}x{} at ({},{}){}{}",
            i + 1,
            monitor.bounds.width,
            monitor.bounds.height,
            monitor.bounds.x,
            monitor.bounds.y,
            monitor.work_area.width,
            monitor.work_area.height,
            monitor.work_area.x,
            monitor.work_area.y,
            primary,
            dpi
        );
    }

    // 3. Check for focused window
    info!("--- Focus ---");
    if let Some(focused) = windows.iter().find(|id| wm.is_focused(**id)) {
        info!("Focused window: {}", wm.describe_window(*focused));
    } else {
        warn!("No focused window found");
    }

    // 4. Classification summary
    info!("--- Classification ---");
    let mut visible = 0;
    let mut minimized = 0;
    let mut maximized = 0;
    let mut topmost = 0;
    let mut non_resizable = 0;
    for id in &windows {
        if wm.is_visible(*id) {
            visible += 1;
        }
        if wm.is_minimized(*id) {
            minimized += 1;
        }
        if wm.is_maximized(*id) {
            maximized += 1;
        }
        if wm.is_topmost(*id) {
            topmost += 1;
        }
        if !wm.is_resizable(*id) {
            non_resizable += 1;
        }
    }
    info!("  Visible: {}", visible);
    info!("  Minimized: {}", minimized);
    info!("  Maximized: {}", maximized);
    info!("  Topmost: {}", topmost);
    info!("  Non-resizable: {}", non_resizable);

    info!("=== Diagnostics complete ===");
}
