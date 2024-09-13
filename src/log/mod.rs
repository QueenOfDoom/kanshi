use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use std::fs;

pub fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    fs::create_dir_all("logs").expect("Failed to create logs directory");
    let log_file_name = format!("logs/{}.log", timestamp);

    fern::Dispatch::new()
        .chain(
            fern::Dispatch::new()
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}] {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S"),
                        colors.color(record.level()),
                        message
                    ))
                })
                .level(log::LevelFilter::Debug)
                .filter(|metadata| metadata.target().contains("kanshi"))
                .chain(std::io::stdout()),
        )
        .chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}][{}] {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        record.target(), // Add the log target (module/crate)
                        message
                    ))
                })
                .level(log::LevelFilter::Info)
                .level_for("kanshi", log::LevelFilter::Debug)
                .level_for("tracing::span", log::LevelFilter::Error)
                .chain(fern::log_file(log_file_name)?),
        )
        .apply()?;

    Ok(())
}
