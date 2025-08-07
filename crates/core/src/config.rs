use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{FileSystemManager, FileSystemResult, errors::FileSystemError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigResponse {
    ConfigRouteResponse(ConfigRouteResponse),
    String(String),
    Str(String),
}

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

#[allow(clippy::unnecessary_wraps)]
fn default_port() -> Option<u16> {
    Some(3001)
}

#[allow(clippy::unnecessary_wraps)]
fn default_host() -> Option<String> {
    Some(String::from("localhost"))
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: default_port(),
            hostname: default_host(),
            routes: HashMap::new(),
        }
    }
}

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
    /// The response body for the route, if applicable
    pub response: ConfigResponse,
}

#[allow(clippy::unnecessary_wraps)]
fn default_method() -> Option<String> {
    Some(String::from("GET"))
}

impl Default for ConfigRoute {
    fn default() -> Self {
        Self {
            method: default_method(),
            description: None,
            headers: None,
            response: ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
                status: default_status(),
                body: Value::Object(Map::new()),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRouteResponse {
    /// The HTTP status code for the response
    #[serde(default = "default_status")]
    pub status: Option<u16>,
    /// The response body, if applicable
    #[serde(default)]
    pub body: Value,
}

#[allow(clippy::unnecessary_wraps)]
fn default_status() -> Option<u16> {
    Some(200)
}

#[derive(Debug, Clone)]
pub struct ConfigManager {
    pub(crate) file_system_manager: FileSystemManager,
    pub config: Config,
}

impl ConfigManager {
    pub fn new(file_system_manager: FileSystemManager) -> Self {
        Self {
            file_system_manager,
            config: Config::default(),
        }
    }

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

    pub fn get_root(&self) -> &PathBuf {
        &self.file_system_manager.root
    }

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
