//! JSON Echo CLI application entry point and command-line interface.
//!
//! This module serves as the main entry point for the JSON Echo CLI application,
//! providing command-line argument parsing, configuration management, and command
//! execution coordination. It handles the initialization and serving functionality
//! of the JSON Echo mock server.
//!
//! ## What
//!
//! The module defines:
//! - `Cli`: Main command-line interface structure with global options
//! - `Commands`: Available subcommands (Init, Serve)
//! - Main function that orchestrates application startup and command execution
//!
//! ## How
//!
//! The application works by:
//! 1. Parsing command-line arguments using clap
//! 2. Determining the working directory based on executable location or config path
//! 3. Setting up filesystem and configuration managers
//! 4. Executing the requested command (init or serve)
//! 5. For serving, loading configuration, populating database, and starting the server
//!
//! ## Why
//!
//! This design enables:
//! - User-friendly command-line interface with clear subcommands
//! - Flexible configuration file handling with relative and absolute paths
//! - Proper error handling and propagation throughout the application
//! - Clean separation between CLI concerns and core business logic
//! - Extensible command structure for future functionality
//!
//! # Examples
//!
//! ```bash
//! # Initialize a new configuration file
//! json-echo init
//!
//! # Serve with default configuration
//! json-echo serve
//!
//! # Serve with custom configuration file
//! json-echo --config ./custom-config.json serve
//!
//! # Serve with custom log level
//! json-echo --log-level debug serve
//! ```

use crate::server::{create_router, run_server};
use clap::{Parser, Subcommand};
use json_echo_core::{
    ConfigManager, Database, FileSystemError, FileSystemManager, FileSystemResult,
};
use std::{env, path::PathBuf};
use tracing::{error, info};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

mod server;

/// Main command-line interface structure for the JSON Echo application.
///
/// The `Cli` struct defines all available command-line options and subcommands
/// for the JSON Echo application. It uses clap for argument parsing and provides
/// global options that apply to all subcommands.
///
/// # Fields
///
/// * `log_level` - Global logging level configuration (default: "info")
/// * `config` - Path to the configuration file (default: "json-echo.json")
/// * `protocol` - Network protocol to use (default: "http")
/// * `command` - The subcommand to execute
///
/// # Examples
///
/// ```rust
/// use clap::Parser;
/// // This would typically be called by clap automatically
/// // let cli = Cli::parse();
/// ```
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Logging level for the application (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value_t = String::from("info"))]
    log_level: String,

    /// Path to configuration file (can be relative or absolute)
    #[arg(long, global = true, default_value_t = String::from("json-echo.json"))]
    config: String,

    /// Network protocol to use for the server
    #[arg(long, global = true, default_value_t = String::from("http"))]
    protocol: String,

    /// The command to execute
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for the JSON Echo CLI application.
///
/// This enum defines all the subcommands that users can execute with the
/// JSON Echo CLI. Each variant represents a different operational mode
/// of the application.
///
/// # Variants
///
/// * `Init` - Initialize a new configuration file with default settings
/// * `Serve` - Start the JSON Echo server with the specified configuration
///
/// # Examples
///
/// ```bash
/// # Initialize command
/// json-echo init
///
/// # Serve command
/// json-echo serve
/// ```
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new JSON Echo configuration file
    ///
    /// Creates a new configuration file with default settings in the current
    /// directory or at the specified path. This command sets up a basic
    /// configuration that can be customized for specific use cases.
    Init,

    /// Start the JSON Echo server
    ///
    /// Loads the specified configuration file and starts the mock server
    /// according to the defined routes and settings. The server will listen
    /// on the configured hostname and port, serving mock responses based
    /// on the route definitions.
    Serve,
}

/// Main entry point for the JSON Echo CLI application.
///
/// This asynchronous function handles the complete lifecycle of the application,
/// from command-line argument parsing to command execution. It sets up the
/// necessary managers, handles path resolution, and coordinates between the
/// CLI interface and the core application logic.
///
/// # Returns
///
/// * `Ok(())` - If the command executed successfully
/// * `Err(FileSystemError)` - If any filesystem or configuration error occurs
///
/// # Errors
///
/// This function can fail if:
/// - The current executable path cannot be determined
/// - The configuration file path is invalid or inaccessible
/// - Filesystem operations fail during initialization or serving
/// - Configuration loading or parsing fails
/// - Server startup fails due to network or configuration issues
///
/// # Behavior
///
/// The function performs the following steps:
/// 1. Parses command-line arguments using clap
/// 2. Determines the working directory based on executable location
/// 3. Handles absolute vs relative configuration file paths
/// 4. Sets up filesystem and configuration managers
/// 5. Executes the appropriate command (Init or Serve)
///
/// For the Init command:
/// - Creates a default configuration
/// - Saves it to the specified location
/// - Reports the created file path
///
/// For the Serve command:
/// - Loads the configuration file
/// - Populates the in-memory database with route definitions
/// - Extracts server settings (hostname, port)
/// - Starts the web server with the configured routes
///
/// # Examples
///
/// This function is called automatically by the Rust runtime and cannot
/// be invoked directly. The application behavior is controlled through
/// command-line arguments:
///
/// ```bash
/// # Initialize with default settings
/// json-echo init
///
/// # Serve with custom configuration
/// json-echo --config /path/to/config.json serve
/// ```
#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> FileSystemResult<()> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let cli = Cli::parse();

    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .with_filter(EnvFilter::new(cli.log_level));

    tracing_subscriber::registry().with(stdout_layer).init();

    print!(
        "
        ░█▀▀░█▀▀░█░█░█▀█░░░▀▀█░█▀▀░█▀█░█▀█░░░█▀▀░█▀▀░█▀▄░█░█░█▀▀░█▀▄
        ░█▀▀░█░░░█▀█░█░█░░░░░█░▀▀█░█░█░█░█░░░▀▀█░█▀▀░█▀▄░▀▄▀░█▀▀░█▀▄
        ░▀▀▀░▀▀▀░▀░▀░▀▀▀░░░▀▀░░▀▀▀░▀▀▀░▀░▀░░░▀▀▀░▀▀▀░▀░▀░░▀░░▀▀▀░▀░▀
        ⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯
        Version: {VERSION}
        ⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯
        \n
    "
    );

    info!("Starting applying configuration");

    // Get the current executable path to determine the working directory
    let current_exe = env::current_exe()
        .map_err(|_| FileSystemError::Operation("Failed to get current directory".into()))?;

    // Ensure the executable has a parent directory
    if current_exe.parent().is_none() {
        return Err(FileSystemError::Operation(
            "Current executable has no parent directory".into(),
        ));
    }

    // Start with the executable's directory as the default working directory
    let mut current_directory = current_exe.parent().unwrap().to_path_buf();
    let config_file = PathBuf::from(&cli.config);

    // If the config file path is absolute, use its directory as the working directory
    if config_file.is_absolute() {
        let config_dir = config_file.parent().ok_or_else(|| {
            error!("Cannot find the root folder.");
            FileSystemError::Operation("Config file has no parent directory".into())
        })?;
        current_directory = config_dir.to_path_buf();
    }

    // Extract the config file name for loading
    let config_file_name = config_file.file_name().ok_or_else(|| {
        error!("Config file not available.");
        FileSystemError::Operation("Config file has no name".into())
    })?;

    // Set up the filesystem and configuration managers
    let file_system_manager = FileSystemManager::new(Some(current_directory))?;
    let mut config_manager = ConfigManager::new(file_system_manager);

    // Execute the requested command
    match cli.command {
        Commands::Init => {
            info!("Generating default config file.");

            // Create a default configuration and save it
            let config = json_echo_core::Config::default();
            config_manager
                .save_config("json-echo.json", &config)
                .await?;

            info!(
                "Configuration file created at: {}",
                config_manager.get_root().join("json-echo.json").display()
            );
        }
        Commands::Serve => {
            info!("Loading config file.");

            // Load the configuration file
            config_manager
                .load_config(config_file_name.display().to_string().as_str())
                .await?;

            info!("Populating in-memory database.");

            // Populate the in-memory database with route configurations
            let mut db = Database::new();
            db.populate(config_manager.config.routes.clone());

            // Extract server configuration with defaults
            let hostname_string = config_manager
                .config
                .hostname
                .clone()
                .unwrap_or_else(|| "localhost".to_string());
            let hostname = hostname_string.as_str();

            let port_string = config_manager.config.port.unwrap_or(3001).to_string();
            let port = port_string.as_str();

            // Start the server with the configured routes and settings
            run_server(hostname, port, create_router(db)).await?;
        }
    }

    Ok(())
}
