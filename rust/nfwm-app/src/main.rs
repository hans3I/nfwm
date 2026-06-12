use clap::{Parser, Subcommand};
use nfwm_core::commands::Action;
use nfwm_core::settings::{
    default_settings_jsonc, render_settings_jsonc, Settings, SettingsMigration,
};
use nfwm_core::tiling::TilingService;
use nfwm_core::traits::{DisplayProvider, WindowProvider};
use nfwm_core::window::DiscoveryService;
use nfwm_win32::display::Win32DisplayManager;
use nfwm_win32::window::{Win32PlacementProvider, Win32WindowManager};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::panic;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

const DETACHED_PROCESS: u32 = 0x0000_0008;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

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
    /// Send an action to the running runtime
    Action { name: String },
    /// Reload the runtime configuration
    Reload,
    /// Stop the running runtime
    Stop,
    /// Report current runtime status
    Status,
    /// Diagnostic: enumerate windows, displays, and test Win32 APIs
    Diagnose,
    /// Run in shadow mode (read-only, no window movement)
    Shadow,
    #[command(hide = true)]
    /// Internal background runtime entrypoint
    Run,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeStatus {
    pid: u32,
    running: bool,
    config_path: String,
    effective_hotkey_count: usize,
    last_reload_result: String,
    last_error: Option<String>,
}

impl RuntimeStatus {
    fn new(config_path: &Path, settings: &Settings) -> Self {
        Self {
            pid: std::process::id(),
            running: true,
            config_path: config_path.display().to_string(),
            effective_hotkey_count: settings.effective_hotkey_count(),
            last_reload_result: "startup-ok".to_string(),
            last_error: None,
        }
    }
}

fn main() {
    let tracing_paths = RuntimePaths::discover().ok();
    init_tracing(tracing_paths.as_ref());
    let cli = Cli::parse();

    let result = match cli.command {
        None => start_or_attach_runtime(),
        Some(Commands::Run) => run_runtime(),
        Some(Commands::Action { name }) => queue_action(&name),
        Some(Commands::Reload) => signal_runtime("reload.signal"),
        Some(Commands::Stop) => signal_runtime("stop.signal"),
        Some(Commands::Status) => print_status(),
        Some(Commands::Diagnose) => {
            run_diagnostics();
            Ok(())
        }
        Some(Commands::Shadow) => {
            run_shadow_mode();
            Ok(())
        }
    };

    if let Err(err) = result {
        error!("{}", err);
        std::process::exit(1);
    }
}

fn start_or_attach_runtime() -> Result<(), String> {
    let paths = RuntimePaths::discover()?;
    ensure_runtime_dirs(&paths)?;
    log_bootstrap_outcome(ensure_bootstrap_config(&paths)?);

    if paths.lock.exists() {
        info!("nfwm runtime already present; pinging existing runtime");
        return Ok(());
    }

    let exe = std::env::current_exe().map_err(|e| format!("failed to locate current exe: {e}"))?;
    Command::new(exe)
        .arg("run")
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("failed to start background runtime: {e}"))?;

    info!("nfwm runtime started in background");
    Ok(())
}

fn run_runtime() -> Result<(), String> {
    let paths = RuntimePaths::discover()?;
    ensure_runtime_dirs(&paths)?;
    log_bootstrap_outcome(ensure_bootstrap_config(&paths)?);

    let _lock = acquire_lock(&paths.lock)?;
    let parsed = load_settings(&paths.config)?;
    let mut settings = parsed.settings;
    let wm = Win32WindowManager::new();
    let dm = Win32DisplayManager::new();
    let placement = Win32PlacementProvider::new();
    let mut discovery = DiscoveryService::new();
    let mut tiling = TilingService::with_work_area(primary_work_area(&dm));
    tiling.start();

    let mut status = RuntimeStatus::new(&paths.config, &settings);
    if !parsed.notes.is_empty() {
        status.last_reload_result = "startup-normalized".to_string();
        for note in &parsed.notes {
            info!("config note: {}", note);
        }
    }
    write_status(&paths.status, &status)?;

    info!("nfwm runtime loop started");
    loop {
        if paths.stop.exists() {
            let _ = fs::remove_file(&paths.stop);
            info!("stop signal received");
            break;
        }

        if paths.reload.exists() {
            let _ = fs::remove_file(&paths.reload);
            match load_settings(&paths.config) {
                Ok(new_settings) => {
                    settings = new_settings.settings;
                    status.effective_hotkey_count = settings.effective_hotkey_count();
                    status.last_reload_result = if new_settings.notes.is_empty() {
                        "reload-ok".to_string()
                    } else {
                        "reload-normalized".to_string()
                    };
                    status.last_error = None;
                    info!(
                        "runtime reloaded config; {} effective hotkeys",
                        status.effective_hotkey_count
                    );
                    for note in &new_settings.notes {
                        info!("config note: {}", note);
                    }
                }
                Err(err) => {
                    status.last_reload_result = "reload-failed".to_string();
                    status.last_error = Some(err.clone());
                    warn!("reload failed; keeping last good config: {}", err);
                }
            }
            write_status(&paths.status, &status)?;
        }

        process_action_queue(&paths.actions, &mut tiling, &wm, &placement, &dm);
        tick_runtime(&mut discovery, &mut tiling, &wm, &placement, &dm);
        write_status(&paths.status, &status)?;
        thread::sleep(Duration::from_millis(settings.behavior.poll_interval_ms));
    }

    status.running = false;
    write_status(&paths.status, &status)?;
    let _ = fs::remove_file(&paths.lock);
    info!("nfwm runtime stopped");
    Ok(())
}

fn queue_action(name: &str) -> Result<(), String> {
    let action: Action = name.parse().map_err(|e| format!("{e}"))?;
    let paths = RuntimePaths::discover()?;
    ensure_runtime_dirs(&paths)?;
    if !paths.lock.exists() {
        return Err("runtime is not running".to_string());
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("clock error: {e}"))?
        .as_millis();
    let path = paths
        .actions
        .join(format!("{timestamp}-{}.cmd", action.name()));
    fs::write(&path, action.name()).map_err(|e| format!("failed to queue action: {e}"))?;
    info!("queued action '{}'", action.name());
    Ok(())
}

fn signal_runtime(file_name: &str) -> Result<(), String> {
    let paths = RuntimePaths::discover()?;
    ensure_runtime_dirs(&paths)?;
    if !paths.lock.exists() {
        return Err("runtime is not running".to_string());
    }
    fs::write(paths.runtime_dir.join(file_name), b"")
        .map_err(|e| format!("failed to signal runtime: {e}"))?;
    info!("sent {}", file_name);
    Ok(())
}

fn print_status() -> Result<(), String> {
    let paths = RuntimePaths::discover()?;
    if !paths.status.exists() {
        println!("nfwm status: not running");
        return Ok(());
    }

    let content =
        fs::read_to_string(&paths.status).map_err(|e| format!("failed to read status: {e}"))?;
    let status: RuntimeStatus =
        serde_json::from_str(&content).map_err(|e| format!("failed to parse status: {e}"))?;
    println!("running: {}", status.running);
    println!("pid: {}", status.pid);
    println!("config: {}", status.config_path);
    println!("effective hotkeys: {}", status.effective_hotkey_count);
    println!("last reload result: {}", status.last_reload_result);
    if let Some(error) = status.last_error {
        println!("last error: {}", error);
    }
    Ok(())
}

fn tick_runtime(
    discovery: &mut DiscoveryService,
    tiling: &mut TilingService,
    wm: &Win32WindowManager,
    placement: &Win32PlacementProvider,
    dm: &Win32DisplayManager,
) {
    tiling.set_work_area(primary_work_area(dm));
    let (_, _) = discovery.refresh(wm, &|| wm.enumerate_windows());
    let windows = discovery.registry().ids();
    let _ = tiling.sync_window_set(&windows);
    tiling.discover(wm, &windows);
    if let Some(focused) = discovery.registry().focused() {
        if let Some(node) = tiling.workspace().find_node(focused) {
            tiling.workspace_mut().set_focus(node);
        }
    }
    tiling.refresh();
    let results = tiling.apply_layout(placement, false);
    for r in &results {
        if !r.success {
            warn!(
                "[DEBUG-place] failed to place {:?} -> {}x{} at ({},{}): {:?} [{}]",
                r.window_id,
                r.rect.width,
                r.rect.height,
                r.rect.x,
                r.rect.y,
                r.error,
                wm.describe_window(r.window_id)
            );
        }
    }
}

fn process_action_queue(
    action_dir: &Path,
    tiling: &mut TilingService,
    wm: &Win32WindowManager,
    placement: &Win32PlacementProvider,
    dm: &Win32DisplayManager,
) {
    let Ok(entries) = fs::read_dir(action_dir) else {
        return;
    };

    let mut paths = entries
        .filter_map(|e| e.ok().map(|x| x.path()))
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                warn!("failed to read action command {:?}: {}", path, err);
                let _ = fs::remove_file(&path);
                continue;
            }
        };

        match content.trim().parse::<Action>() {
            Ok(action) => {
                if let Err(err) = dispatch_action(tiling, action) {
                    warn!("action '{}' failed: {}", action, err);
                }
                tick_runtime_once_after_action(tiling, wm, placement, dm);
            }
            Err(err) => warn!("invalid queued action in {:?}: {}", path, err),
        }
        let _ = fs::remove_file(&path);
    }
}

fn tick_runtime_once_after_action(
    tiling: &mut TilingService,
    wm: &Win32WindowManager,
    placement: &Win32PlacementProvider,
    dm: &Win32DisplayManager,
) {
    tiling.set_work_area(primary_work_area(dm));
    let windows = wm.enumerate_windows();
    let _ = tiling.sync_window_set(&windows);
    tiling.discover(wm, &windows);
    tiling.refresh();
    let _ = tiling.apply_layout(placement, false);
}

fn dispatch_action(tiling: &mut TilingService, action: Action) -> Result<(), String> {
    use nfwm_core::types::{Direction, PanelOrientation};

    match action {
        Action::SplitHorizontal => tiling.split(false).map_err(|e| e.to_string()),
        Action::SplitVertical => tiling.split(true).map_err(|e| e.to_string()),
        Action::Stack => tiling.stack().map_err(|e| e.to_string()),
        Action::Float => tiling.float_window().map_err(|e| e.to_string()),
        Action::MoveFocus(direction) => tiling.move_focus(direction).map_err(|e| e.to_string()),
        Action::Swap(direction) => tiling.swap(direction).map_err(|e| e.to_string()),
        Action::MoveWindow(direction) => tiling.move_window(direction).map_err(|e| e.to_string()),
        Action::PullUp => tiling.pull_up().map_err(|e| e.to_string()),
        Action::Resize(Direction::Left) => tiling
            .resize(PanelOrientation::Horizontal, -50)
            .map_err(|e| e.to_string()),
        Action::Resize(Direction::Right) => tiling
            .resize(PanelOrientation::Horizontal, 50)
            .map_err(|e| e.to_string()),
        Action::Resize(Direction::Up) => tiling
            .resize(PanelOrientation::Vertical, -50)
            .map_err(|e| e.to_string()),
        Action::Resize(Direction::Down) => tiling
            .resize(PanelOrientation::Vertical, 50)
            .map_err(|e| e.to_string()),
        Action::Start => {
            tiling.start();
            Ok(())
        }
        Action::Stop => {
            tiling.stop();
            Ok(())
        }
        Action::Discover => Ok(()),
        Action::Refresh => {
            tiling.refresh();
            Ok(())
        }
        Action::Toggle => {
            tiling.toggle();
            Ok(())
        }
    }
}

fn load_settings(path: &Path) -> Result<SettingsMigration, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("failed to read config: {e}"))?;
    Settings::parse_jsonc_with_notes(&content).map_err(|e| e.to_string())
}

fn ensure_bootstrap_config(paths: &RuntimePaths) -> Result<String, String> {
    if paths.config.exists() {
        return Ok("using existing config.jsonc".to_string());
    }

    if let Some(legacy_path) = find_legacy_settings(paths) {
        match fs::read_to_string(&legacy_path) {
            Ok(content) => match Settings::from_legacy_json(&content) {
                Ok(migrated) => {
                    let mut comments = vec![
                        "nfwm configuration".to_string(),
                        format!(
                            "Migrated from legacy settings at {}.",
                            legacy_path.display()
                        ),
                        "The original legacy settings file was left untouched for rollback safety."
                            .to_string(),
                    ];
                    comments.extend(migrated.notes.iter().cloned());
                    fs::write(
                        &paths.config,
                        render_settings_jsonc(&migrated.settings, &comments),
                    )
                    .map_err(|e| format!("failed to write migrated config: {e}"))?;
                    write_migration_report(paths, &legacy_path, &migrated.notes)?;
                    return Ok(format!(
                        "migrated legacy settings from {}",
                        legacy_path.display()
                    ));
                }
                Err(err) => {
                    fs::write(&paths.config, default_settings_jsonc())
                        .map_err(|e| format!("failed to write default config: {e}"))?;
                    write_ignored_legacy_report(paths, &legacy_path, &err.to_string())?;
                    return Ok(format!(
                        "legacy settings at {} were ignored after parse failure; created default config.jsonc",
                        legacy_path.display()
                    ));
                }
            },
            Err(err) => {
                fs::write(&paths.config, default_settings_jsonc())
                    .map_err(|e| format!("failed to write default config: {e}"))?;
                write_ignored_legacy_report(paths, &legacy_path, &err.to_string())?;
                return Ok(format!(
                    "legacy settings at {} were ignored after read failure; created default config.jsonc",
                    legacy_path.display()
                ));
            }
        }
    }

    fs::write(&paths.config, default_settings_jsonc())
        .map_err(|e| format!("failed to write default config: {e}"))?;
    Ok("created default config.jsonc".to_string())
}

fn write_status(path: &Path, status: &RuntimeStatus) -> Result<(), String> {
    let content = serde_json::to_string_pretty(status)
        .map_err(|e| format!("failed to serialize status: {e}"))?;
    fs::write(path, content).map_err(|e| format!("failed to write status: {e}"))
}

fn acquire_lock(path: &Path) -> Result<fs::File, String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("failed to acquire runtime lock: {e}"))?;
    writeln!(file, "{}", std::process::id())
        .map_err(|e| format!("failed to write runtime lock: {e}"))?;
    Ok(file)
}

fn ensure_runtime_dirs(paths: &RuntimePaths) -> Result<(), String> {
    fs::create_dir_all(&paths.runtime_dir)
        .map_err(|e| format!("failed to create runtime directory: {e}"))?;
    fs::create_dir_all(&paths.actions)
        .map_err(|e| format!("failed to create actions directory: {e}"))
}

struct RuntimePaths {
    runtime_dir: PathBuf,
    config: PathBuf,
    legacy_fancywm_settings: PathBuf,
    legacy_nfwm_settings: PathBuf,
    lock: PathBuf,
    status: PathBuf,
    migration_report: PathBuf,
    log_file: PathBuf,
    stop: PathBuf,
    reload: PathBuf,
    actions: PathBuf,
}

impl RuntimePaths {
    fn discover() -> Result<Self, String> {
        let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA is not set".to_string())?;
        let appdata = PathBuf::from(appdata);
        let runtime_dir = appdata.join("nfwm");
        Ok(Self {
            config: runtime_dir.join("config.jsonc"),
            legacy_fancywm_settings: appdata.join("FancyWM").join("settings.json"),
            legacy_nfwm_settings: runtime_dir.join("settings.json"),
            lock: runtime_dir.join("runtime.lock"),
            status: runtime_dir.join("runtime-status.json"),
            migration_report: runtime_dir.join("migration-report.txt"),
            log_file: runtime_dir.join("nfwm.log"),
            stop: runtime_dir.join("stop.signal"),
            reload: runtime_dir.join("reload.signal"),
            actions: runtime_dir.join("actions"),
            runtime_dir,
        })
    }
}

fn init_tracing(paths: Option<&RuntimePaths>) {
    if let Some(paths) = paths {
        let _ = fs::create_dir_all(&paths.runtime_dir);
        install_panic_hook(paths.runtime_dir.clone());
        rotate_log_if_needed(&paths.log_file);
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&paths.log_file)
        {
            let subscriber = tracing_subscriber::fmt()
                .with_ansi(false)
                .with_writer(move || file.try_clone().expect("clone log file"))
                .finish();
            if tracing::subscriber::set_global_default(subscriber).is_ok() {
                return;
            }
        }
    }

    tracing_subscriber::fmt::init();
}

fn rotate_log_if_needed(path: &Path) {
    const MAX_LOG_BYTES: u64 = 1_048_576;

    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if metadata.len() < MAX_LOG_BYTES {
        return;
    }

    let rotated = path.with_extension("log.1");
    let _ = fs::remove_file(&rotated);
    let _ = fs::rename(path, rotated);
}

fn install_panic_hook(runtime_dir: PathBuf) {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = runtime_dir.join(format!("nfwm-crash-{timestamp}.log"));
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "panic: {info}");
        }
        previous(info);
    }));
}

fn find_legacy_settings(paths: &RuntimePaths) -> Option<PathBuf> {
    [
        paths.legacy_fancywm_settings.clone(),
        paths.legacy_nfwm_settings.clone(),
    ]
    .into_iter()
    .find(|path| path.exists())
}

fn write_migration_report(
    paths: &RuntimePaths,
    legacy_path: &Path,
    notes: &[String],
) -> Result<(), String> {
    let mut report = vec![
        format!("Imported legacy settings from {}", legacy_path.display()),
        "Original file left untouched.".to_string(),
    ];
    report.extend(notes.iter().cloned());
    fs::write(&paths.migration_report, report.join("\n"))
        .map_err(|e| format!("failed to write migration report: {e}"))
}

fn write_ignored_legacy_report(
    paths: &RuntimePaths,
    legacy_path: &Path,
    error: &str,
) -> Result<(), String> {
    let report = [
        format!("Ignored legacy settings from {}", legacy_path.display()),
        format!("Reason: {error}"),
        "Created a fresh config.jsonc instead. The legacy file was left untouched.".to_string(),
    ]
    .join("\n");
    fs::write(&paths.migration_report, report)
        .map_err(|e| format!("failed to write migration report: {e}"))
}

fn log_bootstrap_outcome(message: String) {
    info!("config bootstrap: {message}");
}

fn run_shadow_mode() {
    info!("Shadow mode: discovering windows without moving them");
    let wm = Win32WindowManager::new();
    let dm = Win32DisplayManager::new();
    let mut discovery = DiscoveryService::new();
    let (new, removed) = discovery.refresh(&wm, &|| wm.enumerate_windows());
    info!(
        "Found {} windows ({} new, {} removed)",
        discovery.registry().len(),
        new.len(),
        removed.len()
    );
    for entry in discovery.registry().all() {
        let state = match entry.state {
            nfwm_core::window::classifier::WindowState::Tiled => "tiled",
            nfwm_core::window::classifier::WindowState::Floating => "floating",
            nfwm_core::window::classifier::WindowState::Ignored => "ignored",
        };
        info!(
            "[{}] '{}' (class: {}) -> {}",
            entry.process_id, entry.title, entry.class_name, state
        );
    }

    info!("Shadow mode: simulating tiling layout");
    let mut tiling = TilingService::with_work_area(primary_work_area(&dm));
    tiling.start();
    let windows = discovery.registry().ids();
    let managed = tiling.discover(&wm, &windows);
    info!("Managed {} windows in tiling tree", managed.len());
    if let Some(focused) = discovery.registry().focused() {
        if let Some(node) = tiling.workspace().find_node(focused) {
            tiling.workspace_mut().set_focus(node);
        }
    }
    tiling.refresh();
    if let Some(tree) = tiling.workspace().current_tree() {
        info!("Layout tree:\n{}", tree.visualize());
    }

    let placement = Win32PlacementProvider::new();
    let results = tiling.apply_layout(&placement, true);
    info!("Shadow placements ({} windows):", results.len());
    for r in results.iter().take(10) {
        info!(
            "  {:?} -> {}x{} at ({},{})",
            r.window_id, r.rect.width, r.rect.height, r.rect.x, r.rect.y
        );
    }
    if results.len() > 10 {
        info!("  ... and {} more", results.len() - 10);
    }
    info!("Shadow mode complete. No windows were moved.");
}

fn run_diagnostics() {
    info!("=== nfwm Diagnostics ===");

    info!("--- Windows ---");
    let wm = Win32WindowManager::new();
    let mut discovery = DiscoveryService::new();
    let (new, removed) = discovery.refresh(&wm, &|| wm.enumerate_windows());
    let windows = discovery.registry().ids();
    info!(
        "Found {} top-level windows ({} new, {} removed)",
        windows.len(),
        new.len(),
        removed.len()
    );
    for (i, id) in windows.iter().take(10).enumerate() {
        info!("  {}: {}", i + 1, wm.describe_window(*id));
    }
    if windows.len() > 10 {
        info!("  ... and {} more", windows.len() - 10);
    }

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

    info!("--- Focus ---");
    if let Some(focused) = discovery.registry().focused() {
        info!("Focused window: {}", wm.describe_window(focused));
    } else {
        warn!("No focused window found");
    }

    info!("--- Classification ---");
    let mut tiled = 0;
    let mut floating = 0;
    let mut ignored = 0;
    for entry in discovery.registry().all() {
        match entry.state {
            nfwm_core::window::classifier::WindowState::Tiled => tiled += 1,
            nfwm_core::window::classifier::WindowState::Floating => floating += 1,
            nfwm_core::window::classifier::WindowState::Ignored => ignored += 1,
        }
    }
    info!("  Tiled: {}", tiled);
    info!("  Floating: {}", floating);
    info!("  Ignored: {}", ignored);
    info!(
        "  (Visible: {}, Minimized: {}, Maximized: {}, Topmost: {})",
        windows.iter().filter(|id| wm.is_visible(**id)).count(),
        windows.iter().filter(|id| wm.is_minimized(**id)).count(),
        windows.iter().filter(|id| wm.is_maximized(**id)).count(),
        windows.iter().filter(|id| wm.is_topmost(**id)).count(),
    );

    info!("=== Diagnostics complete ===");
}

fn primary_work_area(dm: &Win32DisplayManager) -> nfwm_core::types::Rectangle {
    dm.work_area(dm.primary_display())
        .unwrap_or_else(|| nfwm_core::types::Rectangle::new(0, 0, 1920, 1080))
}
