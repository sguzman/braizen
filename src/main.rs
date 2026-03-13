use brazen::{BootstrapOptions, BrazenApp, ServoEngineFactory, bootstrap};
use tracing::{error, info};

fn main() {
    if let Err(error) = run() {
        eprintln!("brazen failed to start: {error}");
        error!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let bootstrap = bootstrap(BootstrapOptions { config_path: None }, &ServoEngineFactory)?;
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
    let app_name = config.app.name.clone();

    eframe::run_native(
        &app_name,
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(BrazenApp::new(
                config.clone(),
                shell_state.clone(),
            )))
        }),
    )?;

    Ok(())
}
