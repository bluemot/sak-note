// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::OpenOptions;
use std::path::PathBuf;
use directories::BaseDirs;
use log::LevelFilter;
use simplelog::{ConfigBuilder, WriteLogger};

fn setup_logging() {
    // Get log directory
    let log_dir = if let Some(base) = BaseDirs::new() {
        base.config_dir().join("sak-editor").join("logs")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
            .join(".config")
            .join("sak-editor")
            .join("logs")
    };

    // Create log directory
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
        return;
    }

    // Create log file path with timestamp
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_file = log_dir.join(format!("sak-editor_{}.log", timestamp));

    // Open log file
    let file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open log file: {}", e);
            return;
        }
    };

    // Build log config with target module filtering
    let config = ConfigBuilder::new()
        .set_target_level(LevelFilter::Debug)
        .set_location_level(LevelFilter::Off)
        .build();

    // Initialize logger
    let log_level = match std::env::var("SAK_LOG_LEVEL").as_deref() {
        Ok("trace") => LevelFilter::Trace,
        Ok("debug") => LevelFilter::Debug,
        Ok("info") => LevelFilter::Info,
        Ok("warn") => LevelFilter::Warn,
        Ok("error") => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };

    if let Err(e) = WriteLogger::init(log_level, config, file) {
        eprintln!("Failed to initialize logger: {}", e);
        return;
    }

    log::info!("[main] Logging initialized. Log file: {:?}", log_file);
    log::info!("[main] Log level: {:?}", log_level);
}

fn main() {
    setup_logging();
    log::info!("[main] SAK Editor starting...");
    
    sak_editor::run();
}
