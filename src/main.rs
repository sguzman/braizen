use brazen::automation::start_automation_runtime;
use brazen::cli_cache::run_cache_cli;
use brazen::cli_introspect::run_introspect_cli;
use brazen::{BootstrapOptions, BrazenApp, ServoEngineFactory, bootstrap};
use clap::{Parser, Subcommand};
use tracing::{error, info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to custom config file
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage the browser cache
    Cache {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Introspect and audit a running browser instance
    Introspect {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Cache { args } => {
                if let Err(e) = run_cache_cli(&args) {
                    eprintln!("cache command failed: {e}");
                    std::process::exit(1);
                }
                return;
            }
            Commands::Introspect { args } => {
                if let Err(e) = run_introspect_cli(&args).await {
                    eprintln!("introspect command failed: {e}");
                    std::process::exit(1);
                }
                return;
            }
        }
    }

    if let Err(error) = run(cli.config).await {
        eprintln!("brazen failed to start: {error}");
        error!("{error}");
        std::process::exit(1);
    }
}

async fn run(config_path: Option<std::path::PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let bootstrap = bootstrap(BootstrapOptions { config_path }, &ServoEngineFactory)?;
    info!("starting brazen shell");

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([
                bootstrap.config.window.initial_width,
                bootstrap.config.window.initial_height,
            ])
            .with_title(format!(
                "{} {}",
                bootstrap.config.app.name, bootstrap.config.app.tagline
            )),
        ..Default::default()
    };

    let config = bootstrap.config.clone();
    let shell_state = bootstrap.shell_state;
    let automation = start_automation_runtime(&config, &bootstrap.paths, shell_state.mount_manager.clone());
    let app_name = config.app.name.clone();

    eframe::run_native(
        &app_name,
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(BrazenApp::new(
                config.clone(),
                shell_state.clone(),
                automation,
            )))
        }),
    )?;

    Ok(())
}
