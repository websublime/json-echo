use clap::{Parser, Subcommand};
use json_echo_core::{
    ConfigManager, Database, FileSystemError, FileSystemManager, FileSystemResult,
};
use std::{env, path::PathBuf};

use crate::server::{create_router, run_server};

mod server;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(long, global = true, default_value_t = String::from("info"))]
    log_level: String,

    /// Path to configuration file
    #[arg(long, global = true, default_value_t = String::from("json-echo.json"))]
    config: String,

    #[arg(long, global = true, default_value_t = String::from("http"))]
    protocol: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Serve,
}

#[tokio::main]
async fn main() -> FileSystemResult<()> {
    let cli = Cli::parse();

    let current_exe = env::current_exe()
        .map_err(|_| FileSystemError::Operation("Failed to get current directory".into()))?;

    if current_exe.parent().is_none() {
        return Err(FileSystemError::Operation(
            "Current executable has no parent directory".into(),
        ));
    }

    let mut current_directory = current_exe.parent().unwrap().to_path_buf();
    let config_file = PathBuf::from(&cli.config);

    if config_file.is_absolute() {
        let config_dir = config_file.parent().ok_or_else(|| {
            FileSystemError::Operation("Config file has no parent directory".into())
        })?;
        current_directory = config_dir.to_path_buf();
    }

    let config_file_name = config_file
        .file_name()
        .ok_or_else(|| FileSystemError::Operation("Config file has no name".into()))?;

    let file_system_manager = FileSystemManager::new(Some(current_directory))?;
    let mut config_manager = ConfigManager::new(file_system_manager);

    match cli.command {
        Commands::Init => {
            let config = json_echo_core::Config::default();
            config_manager
                .save_config("json-echo.json", &config)
                .await?;

            println!(
                "Configuration file created at: {}",
                config_manager.get_root().join("json-echo.json").display()
            );
        }
        Commands::Serve => {
            config_manager
                .load_config(config_file_name.display().to_string().as_str())
                .await?;

            let mut db = Database::new();
            db.populate(config_manager.config.routes.clone());

            let hostname_string = config_manager
                .config
                .hostname
                .clone()
                .unwrap_or_else(|| "localhost".to_string());
            let hostname = hostname_string.as_str();

            let port_string = config_manager.config.port.unwrap_or(3001).to_string();
            let port = port_string.as_str();

            run_server(hostname, port, create_router(db)).await?;
        }
    }

    Ok(())
}
