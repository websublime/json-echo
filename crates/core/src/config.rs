//! Configuration module for managing JSON Echo application settings and route definitions.
//!
//! This module provides the core configuration management functionality for the JSON Echo
//! application. It handles loading, parsing, and managing configuration files that define
//! server settings, route configurations, and response specifications.
//!
//! ## What
//!
//! The module defines several key structures:
//! - `Config`: Main configuration container for server settings and routes
//! - `ConfigRoute`: Individual route configuration with HTTP method, headers, and response data
//! - `ConfigResponse`: Enum representing different types of response configurations
//! - `ConfigRouteResponse`: Structured response data with status codes and body content
//! - `ConfigManager`: Manager for loading, saving, and processing configuration files
//!
//! ## How
//!
//! The configuration system works by:
//! 1. Loading JSON configuration files from the filesystem
//! 2. Deserializing configuration data into structured types using serde
//! 3. Processing external file references for route responses
//! 4. Providing access to configuration data through a centralized manager
//! 5. Supporting both inline and file-based response definitions
//!
//! ## Why
//!
//! This design enables:
//! - Flexible configuration through JSON files
//! - Separation of route definitions from response data
//! - Type-safe access to configuration parameters
//! - Automatic file discovery and loading
//! - Support for complex nested configuration structures
//!
//! # Examples
//!
//! ```rust
//! use json_echo_core::{ConfigManager, FileSystemManager};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs_manager = FileSystemManager::new(None)?;
//! let mut config_manager = ConfigManager::new(fs_manager);
//!
//! // Load configuration from a JSON file
//! config_manager.load_config("config.json").await?;
//!
//! // Access server configuration
//! let port = config_manager.config.port.unwrap_or(3000);
//! let hostname = config_manager.config.hostname.as_deref().unwrap_or("localhost");
//!
//! println!("Server will run on {}:{}", hostname, port);
//! # Ok(())
//! # }
//! ```

use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{FileSystemManager, FileSystemResult, errors::FileSystemError};

/// Represents different types of response configurations for routes.
///
/// This enum allows routes to define their responses in multiple ways:
/// either as structured response objects, plain strings, or file references.
/// The untagged serde attribute enables automatic deserialization based on
/// the JSON structure without requiring explicit type indicators.
///
/// # Variants
///
/// * `ConfigRouteResponse` - A structured response with status code and body
/// * `String` - A plain string response (typically file paths or simple text)
/// * `Str` - Another string variant for flexibility in configuration formats
///
/// # Examples
///
/// ```rust
/// use json_echo_core::{ConfigResponse, ConfigRouteResponse};
/// use serde_json::Value;
///
/// // Structured response
/// let structured = ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
///     status: Some(200),
///     body: Value::Null,
/// });
///
/// // String response (often used for file references)
/// let file_ref = ConfigResponse::String("data/users.json".to_string());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigResponse {
    /// A structured response configuration with status code and JSON body
    ConfigRouteResponse(ConfigRouteResponse),
    /// A string response, often used for file path references
    String(String),
    /// Alternative string representation for configuration flexibility
    Str(String),
}

/// Main configuration structure containing server settings and route definitions.
///
/// The `Config` struct serves as the root configuration object that contains
/// all server-level settings such as port and hostname, as well as a collection
/// of route configurations. It supports JSON deserialization with sensible
/// defaults for missing values.
///
/// # Fields
///
/// * `port` - Optional server port number (defaults to 3001)
/// * `hostname` - Optional server hostname (defaults to "localhost")
/// * `routes` - HashMap of route configurations indexed by route path
///
/// # Examples
///
/// ```rust
/// use json_echo_core::Config;
/// use std::collections::HashMap;
///
/// // Create default configuration
/// let config = Config::default();
/// assert_eq!(config.port, Some(3001));
/// assert_eq!(config.hostname, Some("localhost".to_string()));
/// assert!(config.routes.is_empty());
///
/// // Configuration can be loaded from JSON
/// let json_config = r#"
/// {
///     "port": 8080,
///     "hostname": "0.0.0.0",
///     "routes": {}
/// }
/// "#;
/// let parsed: Config = serde_json::from_str(json_config).unwrap();
/// assert_eq!(parsed.port, Some(8080));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The port on which the server will listen
    #[serde(default = "default_port")]
    pub port: Option<u16>,
    /// The hostname for the server
    #[serde(default = "default_host")]
    pub hostname: Option<String>,
    /// A map of routes, where the key is the route path and the value is the route configuration
    #[serde(default = "HashMap::new")]
    pub routes: HashMap<String, ConfigRoute>,
}

/// Returns the default port number for the server.
///
/// Provides a default port value of 3001 for server configuration.
/// This function is used by serde as the default value provider when
/// the port field is missing from the configuration JSON.
///
/// # Returns
///
/// `Some(3001)` - The default port number wrapped in an Option
#[allow(clippy::unnecessary_wraps)]
fn default_port() -> Option<u16> {
    Some(3001)
}

/// Returns the default hostname for the server.
///
/// Provides a default hostname value of "localhost" for server configuration.
/// This function is used by serde as the default value provider when
/// the hostname field is missing from the configuration JSON.
///
/// # Returns
///
/// `Some("localhost")` - The default hostname string wrapped in an Option
#[allow(clippy::unnecessary_wraps)]
fn default_host() -> Option<String> {
    Some(String::from("localhost"))
}

impl Default for Config {
    /// Creates a new Config instance with default values.
    ///
    /// Provides a default configuration suitable for development environments
    /// with localhost hostname, port 3001, and no predefined routes.
    ///
    /// # Returns
    ///
    /// A new `Config` instance with default server settings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Config;
    ///
    /// let config = Config::default();
    /// assert_eq!(config.port, Some(3001));
    /// assert_eq!(config.hostname, Some("localhost".to_string()));
    /// assert!(config.routes.is_empty());
    /// ```
    fn default() -> Self {
        Self {
            port: default_port(),
            hostname: default_host(),
            routes: HashMap::new(),
        }
    }
}

/// Configuration for an individual route including HTTP method, headers, and response.
///
/// The `ConfigRoute` struct defines how a specific API endpoint should behave,
/// including the HTTP method it responds to, any custom headers, and the response
/// data. It supports flexible response definitions through the ConfigResponse enum.
///
/// # Fields
///
/// * `method` - Optional HTTP method (defaults to "GET")
/// * `description` - Optional human-readable description of the route
/// * `headers` - Optional custom HTTP headers to include in responses
/// * `id_field` - Optional field name to use as unique identifier (defaults to "id")
/// * `results_field` - Optional field name containing results when data is nested
/// * `response` - The response configuration for this route
///
/// # Examples
///
/// ```rust
/// use json_echo_core::{ConfigRoute, ConfigResponse, ConfigRouteResponse};
/// use serde_json::Value;
/// use std::collections::HashMap;
///
/// let mut headers = HashMap::new();
/// headers.insert("Content-Type".to_string(), "application/json".to_string());
///
/// let route = ConfigRoute {
///     method: Some("GET".to_string()),
///     description: Some("Get user list".to_string()),
///     headers: Some(headers),
///     id_field: Some("user_id".to_string()),
///     results_field: Some("data".to_string()),
///     response: ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
///         status: Some(200),
///         body: Value::Null,
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRoute {
    /// The HTTP method for the route (e.g., GET, POST)
    #[serde(default = "default_method")]
    pub method: Option<String>,
    /// The path for the route
    #[serde(default)]
    pub description: Option<String>,
    /// The response status code for the route
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    /// The field in the response body to use as the unique identifier, if applicable
    #[serde(default = "default_id_field")]
    pub id_field: Option<String>,
    /// The field in the response body that contains the results array, if applicable
    #[serde(default)]
    pub results_field: Option<String>,
    /// The response body for the route, if applicable
    pub response: ConfigResponse,
}

/// Returns the default HTTP method for routes.
///
/// Provides a default HTTP method value of "GET" for route configuration.
/// This function is used by serde as the default value provider when
/// the method field is missing from the route configuration JSON.
///
/// # Returns
///
/// `Some("GET")` - The default HTTP method wrapped in an Option
#[allow(clippy::unnecessary_wraps)]
fn default_method() -> Option<String> {
    Some(String::from("GET"))
}

/// Returns the default ID field name for routes.
///
/// Provides a default ID field value of "id" for route configuration.
/// This function is used by serde as the default value provider when
/// the id_field is missing from the route configuration JSON.
///
/// # Returns
///
/// `Some("id")` - The default ID field name wrapped in an Option
#[allow(clippy::unnecessary_wraps)]
fn default_id_field() -> Option<String> {
    Some(String::from("id"))
}

impl Default for ConfigRoute {
    /// Creates a new ConfigRoute instance with default values.
    ///
    /// Provides a default route configuration suitable for GET endpoints
    /// with standard ID field naming and empty response body.
    ///
    /// # Returns
    ///
    /// A new `ConfigRoute` instance with default settings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::ConfigRoute;
    ///
    /// let route = ConfigRoute::default();
    /// assert_eq!(route.method, Some("GET".to_string()));
    /// assert_eq!(route.id_field, Some("id".to_string()));
    /// assert!(route.description.is_none());
    /// ```
    fn default() -> Self {
        Self {
            method: default_method(),
            id_field: default_id_field(),
            description: None,
            results_field: None,
            headers: None,
            response: ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
                status: default_status(),
                body: Value::Object(Map::new()),
            }),
        }
    }
}

/// Structured response configuration with HTTP status and JSON body.
///
/// The `ConfigRouteResponse` struct represents a complete HTTP response
/// specification including the status code and response body content.
/// It provides the most detailed control over route responses.
///
/// # Fields
///
/// * `status` - Optional HTTP status code (defaults to 200)
/// * `body` - JSON response body content
///
/// # Examples
///
/// ```rust
/// use json_echo_core::ConfigRouteResponse;
/// use serde_json::{json, Value};
///
/// // Simple response with default status
/// let response = ConfigRouteResponse {
///     status: None, // Will use default 200
///     body: json!({"message": "Hello, World!"}),
/// };
///
/// // Custom status code response
/// let error_response = ConfigRouteResponse {
///     status: Some(404),
///     body: json!({"error": "Not found"}),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRouteResponse {
    /// The HTTP status code for the response
    #[serde(default = "default_status")]
    pub status: Option<u16>,
    /// The response body, if applicable
    #[serde(default)]
    pub body: Value,
}

/// Returns the default HTTP status code for responses.
///
/// Provides a default status code value of 200 (OK) for response configuration.
/// This function is used by serde as the default value provider when
/// the status field is missing from the response configuration JSON.
///
/// # Returns
///
/// `Some(200)` - The default HTTP status code wrapped in an Option
#[allow(clippy::unnecessary_wraps)]
fn default_status() -> Option<u16> {
    Some(200)
}

/// Manager for loading, processing, and saving configuration files.
///
/// The `ConfigManager` struct provides high-level operations for working with
/// configuration files. It handles file I/O operations, JSON parsing, and
/// processing of external file references within route configurations.
///
/// # Fields
///
/// * `file_system_manager` - Internal filesystem manager for file operations
/// * `config` - The loaded and processed configuration data
///
/// # Examples
///
/// ```rust
/// use json_echo_core::{ConfigManager, FileSystemManager};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs_manager = FileSystemManager::new(None)?;
/// let mut config_manager = ConfigManager::new(fs_manager);
///
/// // Load and process configuration
/// config_manager.load_config("app-config.json").await?;
///
/// // Access configuration
/// let routes = &config_manager.config.routes;
/// println!("Loaded {} routes", routes.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Internal filesystem manager for handling file operations
    pub(crate) file_system_manager: FileSystemManager,
    /// The loaded and processed configuration data
    pub config: Config,
}

impl ConfigManager {
    /// Creates a new ConfigManager instance with the provided filesystem manager.
    ///
    /// Initializes a new configuration manager with a default empty configuration
    /// and the specified filesystem manager for handling file operations.
    ///
    /// # Parameters
    ///
    /// * `file_system_manager` - The filesystem manager to use for file operations
    ///
    /// # Returns
    ///
    /// A new `ConfigManager` instance ready for configuration loading
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::{ConfigManager, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs_manager = FileSystemManager::new(None)?;
    /// let config_manager = ConfigManager::new(fs_manager);
    ///
    /// // Manager is ready to load configurations
    /// assert!(config_manager.config.routes.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(file_system_manager: FileSystemManager) -> Self {
        Self {
            file_system_manager,
            config: Config::default(),
        }
    }

    /// Loads and processes a configuration file from the filesystem.
    ///
    /// This method loads a JSON configuration file, parses it into a Config struct,
    /// validates that routes are present, and processes any external file references
    /// in route responses. It replaces any existing configuration data.
    ///
    /// # Parameters
    ///
    /// * `relative_file_path` - Path to the configuration file relative to the filesystem root
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration was loaded and processed successfully
    /// * `Err(FileSystemError)` - If file loading, parsing, or validation fails
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - The configuration file cannot be read
    /// - The JSON content is malformed or invalid
    /// - The configuration contains no routes
    /// - Referenced external files cannot be loaded
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::{ConfigManager, FileSystemManager};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs_manager = FileSystemManager::new(None)?;
    /// let mut config_manager = ConfigManager::new(fs_manager);
    ///
    /// // Load configuration from JSON file
    /// match config_manager.load_config("config.json").await {
    ///     Ok(()) => println!("Configuration loaded successfully"),
    ///     Err(e) => eprintln!("Failed to load configuration: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_config(&mut self, relative_file_path: &str) -> FileSystemResult<()> {
        let file_content: Vec<u8> = self
            .file_system_manager
            .load_file(relative_file_path)
            .await?;
        self.config = serde_json::from_slice(&file_content).map_err(FileSystemError::from)?;

        if self.config.routes.is_empty() {
            return Err(FileSystemError::Operation(
                "Configuration routes are empty or invalid".into(),
            ));
        }

        self.populate_config().await?;

        Ok(())
    }

    /// Processes route configurations to resolve external file references.
    ///
    /// This internal method iterates through all route configurations and loads
    /// external JSON files referenced in string-type responses. It replaces
    /// string file references with the actual loaded configuration data.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all external files were processed successfully
    /// * `Err(FileSystemError)` - If any external file cannot be loaded or parsed
    ///
    /// # Behavior
    ///
    /// - Only processes routes with `ConfigResponse::String` responses
    /// - Loads external files relative to the filesystem root
    /// - Replaces string references with parsed `ConfigRouteResponse` objects
    /// - Validates that referenced routes still exist after processing
    async fn populate_config(&mut self) -> FileSystemResult<()> {
        let routes = self.config.routes.clone();

        for (path, route) in routes {
            if let ConfigResponse::String(route_file) = route.response {
                let route_file = self.get_root().join(route_file);
                let route_content = self
                    .file_system_manager
                    .load_file(route_file.to_string_lossy().as_ref())
                    .await?;
                let route_config: ConfigRouteResponse =
                    serde_json::from_slice(&route_content).map_err(FileSystemError::from)?;
                self.config
                    .routes
                    .iter_mut()
                    .find(|(p, _)| p.as_str() == path.as_str())
                    .map(|(_, r)| {
                        r.response = ConfigResponse::ConfigRouteResponse(route_config);
                    })
                    .ok_or_else(|| FileSystemError::Operation(format!("Route {path} not found")))?;
            }
        }
        Ok(())
    }

    /// Saves a configuration to a file on the filesystem.
    ///
    /// Serializes the provided configuration to JSON format and writes it to
    /// the specified file path. This method can be used to persist configuration
    /// changes or create new configuration files.
    ///
    /// # Parameters
    ///
    /// * `relative_file_path` - Path where the configuration should be saved, relative to filesystem root
    /// * `config` - The configuration object to serialize and save
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration was saved successfully
    /// * `Err(FileSystemError)` - If serialization or file writing fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::{ConfigManager, FileSystemManager, Config};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs_manager = FileSystemManager::new(None)?;
    /// let config_manager = ConfigManager::new(fs_manager);
    ///
    /// let config = Config::default();
    /// config_manager.save_config("backup-config.json", &config).await?;
    /// println!("Configuration saved successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_config(
        &self,
        relative_file_path: &str,
        config: &Config,
    ) -> FileSystemResult<()> {
        let file_content = serde_json::to_vec(config).map_err(FileSystemError::from)?;
        self.file_system_manager
            .save_file(relative_file_path, file_content)
            .await
    }

    /// Returns the root directory path used by the filesystem manager.
    ///
    /// Provides access to the base directory path that serves as the root
    /// for all relative file operations performed by this configuration manager.
    ///
    /// # Returns
    ///
    /// A reference to the root directory PathBuf
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::{ConfigManager, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs_manager = FileSystemManager::new(None)?;
    /// let config_manager = ConfigManager::new(fs_manager);
    ///
    /// let root_path = config_manager.get_root();
    /// println!("Working directory: {}", root_path.display());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_root(&self) -> &PathBuf {
        &self.file_system_manager.root
    }

    /// Attempts to find a configuration file using common naming patterns.
    ///
    /// Searches for configuration files using standard naming conventions
    /// in the filesystem root directory. This method checks for multiple
    /// common configuration file names and returns the first one found.
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - Path to the first configuration file found
    /// * `None` - If no configuration files are found using standard names
    ///
    /// # Behavior
    ///
    /// Checks for files in this order:
    /// 1. `db.json`
    /// 2. `.db.json`
    /// 3. `json-echo.json`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::{ConfigManager, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs_manager = FileSystemManager::new(None)?;
    /// let config_manager = ConfigManager::new(fs_manager);
    ///
    /// match config_manager.get_config_file_path() {
    ///     Some(path) => println!("Found config file: {}", path.display()),
    ///     None => println!("No standard config file found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_config_file_path(&self) -> Option<PathBuf> {
        let mock_files = ["db.json", ".db.json", "json-echo.json"];
        for mock_file in &mock_files {
            let path = self.file_system_manager.root.join(mock_file);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}
