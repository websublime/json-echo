//! Comprehensive test suite for the config module.
//!
//! This module contains extensive tests for all functionality exposed by the config module,
//! including Config, ConfigRoute, ConfigResponse, BodyResponse, and ConfigManager. Tests are
//! designed to verify correct behavior, serialization/deserialization, error handling, and
//! edge cases across different scenarios.
//!
//! ## What
//!
//! The test suite covers:
//! - Config structure creation, defaults, and serialization/deserialization
//! - ConfigRoute structure with various response types and headers
//! - BodyResponse variants and their access methods
//! - ConfigResponse handling of structured and string responses
//! - ConfigManager functionality including loading, saving, and external file resolution
//! - Error handling for invalid configurations and missing files
//! - External file reference resolution and processing
//!
//! ## How
//!
//! Tests use tempfile crate to create isolated test environments that don't interfere
//! with the actual filesystem. Each test creates its own temporary directory structure
//! and verifies the expected behavior without side effects. JSON serialization and
//! deserialization are thoroughly tested to ensure data integrity.
//!
//! ## Why
//!
//! Comprehensive testing ensures:
//! - Reliability of configuration management operations
//! - Proper JSON serialization/deserialization behavior
//! - Correct handling of external file references
//! - Robust error handling for various failure scenarios
//! - Data integrity across configuration save/load cycles
use json_echo_core::FileSystemError;
use json_echo_core::{
    BodyResponse, Config, ConfigManager, ConfigResponse, ConfigRoute, ConfigRouteResponse,
    FileSystemManager,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::Path,
};
use tempfile::TempDir;

/// Sets up a temporary directory for testing.
///
/// Creates a new temporary directory that will be automatically cleaned up
/// when the returned TempDir is dropped. This ensures test isolation and
/// prevents test artifacts from affecting subsequent test runs.
///
/// # Returns
///
/// A TempDir instance representing the temporary test directory
///
/// # Examples
///
/// ```rust
/// let temp_dir = setup_test_dir();
/// let temp_path = temp_dir.path();
/// // Use temp_path for test operations
/// ```
fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory for test")
}

/// Creates a test file with the specified content in the given directory.
///
/// Helper function that creates a file with the provided content at the
/// specified path within a directory. Used to set up test scenarios with
/// predefined file structures.
///
/// # Parameters
///
/// * `dir` - The directory where the file should be created
/// * `file_name` - The name of the file to create
/// * `content` - The content to write to the file
///
/// # Examples
///
/// ```rust
/// let temp_dir = setup_test_dir();
/// create_test_file(temp_dir.path(), "config.json", b"{\"test\": true}");
/// ```
fn create_test_file(dir: &Path, file_name: &str, content: &[u8]) {
    let file_path = dir.join(file_name);
    let mut file = File::create(file_path).expect("Failed to create test file");
    file.write_all(content)
        .expect("Failed to write content to test file");
    file.sync_all().expect("Failed to sync test file");
}

mod config_structure_tests {
    use super::*;

    /// Tests creation of Config with default values.
    ///
    /// Verifies that the Config::default() creates a configuration with
    /// appropriate default values for all fields.
    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert_eq!(config.port, Some(3001), "Default port should be 3001");
        assert_eq!(
            config.hostname,
            Some("localhost".to_string()),
            "Default hostname should be 'localhost'"
        );
        assert!(
            config.static_folder.is_none(),
            "Default static_folder should be None"
        );
        assert_eq!(
            config.static_route, "/static",
            "Default static_route should be '/static'"
        );
        assert!(config.routes.is_empty(), "Default routes should be empty");
    }

    /// Tests Config deserialization from JSON with all fields.
    ///
    /// Verifies that a complete JSON configuration can be correctly
    /// deserialized into a Config struct with all field values preserved.
    #[test]
    fn test_config_deserialization_complete() {
        let json_config = json!({
            "port": 8080,
            "hostname": "0.0.0.0",
            "static_folder": "public",
            "static_route": "/assets",
            "routes": {
                "/api/users": {
                    "method": "GET",
                    "description": "Get all users",
                    "headers": {
                        "Content-Type": "application/json"
                    },
                    "id_field": "user_id",
                    "results_field": "data",
                    "response": {
                        "status": 200,
                        "body": {"users": []}
                    }
                }
            }
        });

        let config: Config = serde_json::from_value(json_config).expect("Should deserialize");

        assert_eq!(config.port, Some(8080), "Port should be 8080");
        assert_eq!(
            config.hostname,
            Some("0.0.0.0".to_string()),
            "Hostname should be '0.0.0.0'"
        );
        assert_eq!(
            config.static_folder,
            Some("public".to_string()),
            "Static folder should be 'public'"
        );
        assert_eq!(
            config.static_route, "/assets",
            "Static route should be '/assets'"
        );
        assert_eq!(config.routes.len(), 1, "Should have one route");

        let route = config.routes.get("/api/users").expect("Route should exist");
        assert_eq!(
            route.method,
            Some("GET".to_string()),
            "Route method should be GET"
        );
        assert_eq!(
            route.description,
            Some("Get all users".to_string()),
            "Route description should match"
        );
        assert!(route.headers.is_some(), "Headers should be present");
        assert_eq!(
            route.id_field,
            Some("user_id".to_string()),
            "ID field should be 'user_id'"
        );
        assert_eq!(
            route.results_field,
            Some("data".to_string()),
            "Results field should be 'data'"
        );
    }

    /// Tests Config deserialization with minimal JSON.
    ///
    /// Verifies that a minimal JSON configuration uses default values
    /// for missing fields and still creates a valid Config.
    #[test]
    fn test_config_deserialization_minimal() {
        let json_config = json!({
            "routes": {}
        });

        let config: Config = serde_json::from_value(json_config).expect("Should deserialize");

        assert_eq!(config.port, Some(3001), "Should use default port");
        assert_eq!(
            config.hostname,
            Some("localhost".to_string()),
            "Should use default hostname"
        );
        assert!(
            config.static_folder.is_none(),
            "Should use default static_folder"
        );
        assert_eq!(
            config.static_route, "/static",
            "Should use default static_route"
        );
        assert!(config.routes.is_empty(), "Routes should be empty");
    }

    /// Tests Config serialization to JSON.
    ///
    /// Verifies that a Config struct can be correctly serialized to JSON
    /// and maintains all field values after serialization.
    #[test]
    fn test_config_serialization() {
        let mut routes = HashMap::new();
        routes.insert(
            "/test".to_string(),
            ConfigRoute {
                method: Some("POST".to_string()),
                description: Some("Test route".to_string()),
                headers: None,
                id_field: Some("id".to_string()),
                results_field: None,
                response: ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
                    status: Some(201),
                    body: BodyResponse::Value(json!({"message": "created"})),
                }),
            },
        );

        let config = Config {
            port: Some(9000),
            hostname: Some("example.com".to_string()),
            static_folder: Some("assets".to_string()),
            static_route: "/files".to_string(),
            routes,
        };

        let serialized = serde_json::to_value(&config).expect("Should serialize");

        assert_eq!(
            serialized["port"], 9000,
            "Port should be serialized correctly"
        );
        assert_eq!(
            serialized["hostname"], "example.com",
            "Hostname should be serialized correctly"
        );
        assert_eq!(
            serialized["static_folder"], "assets",
            "Static folder should be serialized correctly"
        );
        assert_eq!(
            serialized["static_route"], "/files",
            "Static route should be serialized correctly"
        );
        assert!(
            serialized["routes"].is_object(),
            "Routes should be serialized as object"
        );
    }
}

mod config_route_tests {
    use super::*;

    /// Tests ConfigRoute default values.
    ///
    /// Verifies that ConfigRoute::default() creates a route configuration
    /// with appropriate default values for all fields.
    #[test]
    fn test_config_route_default() {
        let route = ConfigRoute::default();

        assert_eq!(
            route.method,
            Some("GET".to_string()),
            "Default method should be GET"
        );
        assert!(
            route.description.is_none(),
            "Default description should be None"
        );
        assert!(route.headers.is_none(), "Default headers should be None");
        assert_eq!(
            route.id_field,
            Some("id".to_string()),
            "Default id_field should be 'id'"
        );
        assert!(
            route.results_field.is_none(),
            "Default results_field should be None"
        );

        // Verify default response structure
        match &route.response {
            ConfigResponse::ConfigRouteResponse(response) => {
                assert_eq!(response.status, Some(200), "Default status should be 200");
                match &response.body {
                    BodyResponse::Value(Value::Object(map)) => {
                        assert!(map.is_empty(), "Default body should be empty object");
                    }
                    _ => panic!("Default body should be empty JSON object"),
                }
            }
            _ => panic!("Default response should be ConfigRouteResponse"),
        }
    }

    /// Tests ConfigRoute deserialization from JSON.
    ///
    /// Verifies that a JSON route configuration can be correctly
    /// deserialized into a ConfigRoute struct.
    #[test]
    fn test_config_route_deserialization() {
        let json_route = json!({
            "method": "POST",
            "description": "Create new item",
            "headers": {
                "Authorization": "Bearer token",
                "Content-Type": "application/json"
            },
            "id_field": "item_id",
            "results_field": "items",
            "response": {
                "status": 201,
                "body": {
                    "success": true,
                    "message": "Item created"
                }
            }
        });

        let route: ConfigRoute = serde_json::from_value(json_route).expect("Should deserialize");

        assert_eq!(
            route.method,
            Some("POST".to_string()),
            "Method should be POST"
        );
        assert_eq!(
            route.description,
            Some("Create new item".to_string()),
            "Description should match"
        );

        let headers = route.headers.expect("Headers should be present");
        assert_eq!(headers.len(), 2, "Should have 2 headers");
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer token".to_string()),
            "Authorization header should match"
        );

        assert_eq!(
            route.id_field,
            Some("item_id".to_string()),
            "ID field should be 'item_id'"
        );
        assert_eq!(
            route.results_field,
            Some("items".to_string()),
            "Results field should be 'items'"
        );

        match &route.response {
            ConfigResponse::ConfigRouteResponse(response) => {
                assert_eq!(response.status, Some(201), "Status should be 201");
                match &response.body {
                    BodyResponse::Value(value) => {
                        assert_eq!(value["success"], true, "Body should contain success field");
                        assert_eq!(
                            value["message"], "Item created",
                            "Body should contain message field"
                        );
                    }
                    _ => panic!("Body should be JSON value"),
                }
            }
            _ => panic!("Response should be ConfigRouteResponse"),
        }
    }
}

mod body_response_tests {
    use super::*;

    /// Tests BodyResponse Value variant functionality.
    ///
    /// Verifies that BodyResponse::Value correctly handles JSON values
    /// and provides appropriate access methods.
    #[test]
    fn test_body_response_value() {
        let json_value = json!({"key": "value", "number": 42});
        let body = BodyResponse::Value(json_value.clone());

        assert!(body.is_value(), "Should be identified as value");
        assert!(!body.is_str(), "Should not be identified as string");
        assert_eq!(body.as_value(), json_value, "Should return original value");
        assert_eq!(body.as_str(), "", "Should return empty string for value");
    }

    /// Tests BodyResponse String variant functionality.
    ///
    /// Verifies that BodyResponse::String correctly handles string values
    /// and provides appropriate access methods.
    #[test]
    fn test_body_response_string() {
        let text = "Hello, World!".to_string();
        let body = BodyResponse::String(text.clone());

        assert!(!body.is_value(), "Should not be identified as value");
        assert!(body.is_str(), "Should be identified as string");
        assert_eq!(
            body.as_value(),
            Value::String(text.clone()),
            "Should return string as JSON value"
        );
        assert_eq!(body.as_str(), &text, "Should return original string");
    }

    /// Tests BodyResponse Str variant functionality.
    ///
    /// Verifies that BodyResponse::Str correctly handles string values
    /// and provides appropriate access methods.
    #[test]
    fn test_body_response_str() {
        let text = "Test string".to_string();
        let body = BodyResponse::Str(text.clone());

        assert!(!body.is_value(), "Should not be identified as value");
        assert!(body.is_str(), "Should be identified as string");
        assert_eq!(
            body.as_value(),
            Value::String(text.clone()),
            "Should return string as JSON value"
        );
        assert_eq!(body.as_str(), &text, "Should return original string");
    }
}

mod config_response_tests {
    use super::*;

    /// Tests ConfigResponse String variant.
    ///
    /// Verifies that ConfigResponse can correctly handle string values,
    /// typically used for file references.
    #[test]
    fn test_config_response_string() {
        let file_path = "data/users.json".to_string();
        let response = ConfigResponse::String(file_path.clone());

        // Test serialization and deserialization
        let serialized = serde_json::to_value(&response).expect("Should serialize");
        assert_eq!(serialized, file_path, "Should serialize as plain string");

        let deserialized: ConfigResponse =
            serde_json::from_value(serialized).expect("Should deserialize");
        match deserialized {
            ConfigResponse::String(path) => {
                assert_eq!(path, file_path, "Should preserve file path");
            }
            _ => panic!("Should deserialize as String variant"),
        }
    }

    /// Tests ConfigResponse structured variant.
    ///
    /// Verifies that ConfigResponse can correctly handle structured
    /// ConfigRouteResponse objects.
    #[test]
    fn test_config_response_structured() {
        let structured_response = ConfigRouteResponse {
            status: Some(404),
            body: BodyResponse::Value(json!({"error": "Not found"})),
        };
        let response = ConfigResponse::ConfigRouteResponse(structured_response.clone());

        // Test serialization
        let serialized = serde_json::to_value(&response).expect("Should serialize");
        assert!(serialized.is_object(), "Should serialize as JSON object");
        assert_eq!(serialized["status"], 404, "Should preserve status code");

        // Test deserialization
        let deserialized: ConfigResponse =
            serde_json::from_value(serialized).expect("Should deserialize");
        match deserialized {
            ConfigResponse::ConfigRouteResponse(response) => {
                assert_eq!(response.status, Some(404), "Should preserve status");
                match response.body {
                    BodyResponse::Value(value) => {
                        assert_eq!(value["error"], "Not found", "Should preserve body content");
                    }
                    _ => panic!("Body should be JSON value"),
                }
            }
            _ => panic!("Should deserialize as ConfigRouteResponse variant"),
        }
    }
}

mod config_manager_tests {
    use super::*;

    /// Tests ConfigManager creation and initialization.
    ///
    /// Verifies that ConfigManager can be created with a filesystem manager
    /// and initializes with default configuration.
    #[tokio::test]
    async fn test_config_manager_new() {
        let temp_dir = setup_test_dir();
        let fs_manager = FileSystemManager::new(Some(temp_dir.path().to_path_buf()))
            .expect("Should create filesystem manager");

        let config_manager = ConfigManager::new(fs_manager);

        assert!(
            config_manager.config.routes.is_empty(),
            "Should start with empty routes"
        );
        // Use canonicalized paths for comparison to handle macOS /private prefix
        let expected_root = temp_dir
            .path()
            .canonicalize()
            .expect("Should canonicalize temp path");
        let actual_root = config_manager
            .get_root()
            .canonicalize()
            .expect("Should canonicalize actual path");
        assert_eq!(actual_root, expected_root, "Should use provided root path");
    }

    /// Tests successful configuration loading from JSON file.
    ///
    /// Verifies that ConfigManager can load and parse a valid JSON
    /// configuration file with route definitions.
    #[tokio::test]
    async fn test_config_manager_load_config_success() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create a valid configuration file
        let config_json = json!({
            "port": 8080,
            "hostname": "0.0.0.0",
            "static_folder": "public",
            "static_route": "/assets",
            "routes": {
                "/api/test": {
                    "method": "GET",
                    "description": "Test endpoint",
                    "response": {
                        "status": 200,
                        "body": {"message": "Hello, World!"}
                    }
                }
            }
        });

        create_test_file(
            temp_path,
            "config.json",
            serde_json::to_string_pretty(&config_json)
                .expect("Should serialize config")
                .as_bytes(),
        );

        let fs_manager = FileSystemManager::new(Some(temp_path.to_path_buf()))
            .expect("Should create filesystem manager");
        let mut config_manager = ConfigManager::new(fs_manager);

        let result = config_manager.load_config("config.json").await;
        assert!(result.is_ok(), "Should load configuration successfully");

        assert_eq!(
            config_manager.config.port,
            Some(8080),
            "Should load port correctly"
        );
        assert_eq!(
            config_manager.config.hostname,
            Some("0.0.0.0".to_string()),
            "Should load hostname correctly"
        );
        assert_eq!(
            config_manager.config.routes.len(),
            1,
            "Should load routes correctly"
        );

        let route = config_manager
            .config
            .routes
            .get("[GET] /api/test")
            .expect("Route should exist");
        assert_eq!(
            route.method,
            Some("GET".to_string()),
            "Route method should be correct"
        );
    }

    /// Tests configuration loading with external file references.
    ///
    /// Verifies that ConfigManager can resolve external file references
    /// in route configurations and load the referenced data.
    #[tokio::test]
    async fn test_config_manager_load_config_with_external_files() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create external route response file
        let external_response = json!({
            "status": 200,
            "body": {
                "users": [
                    {"id": 1, "name": "John"},
                    {"id": 2, "name": "Jane"}
                ]
            }
        });

        create_test_file(
            temp_path,
            "users_response.json",
            serde_json::to_string_pretty(&external_response)
                .expect("Should serialize response")
                .as_bytes(),
        );

        // Create configuration with external file reference
        let config_json = json!({
            "port": 3000,
            "routes": {
                "/api/users": {
                    "method": "GET",
                    "description": "Get all users",
                    "response": "users_response.json"
                }
            }
        });

        create_test_file(
            temp_path,
            "config.json",
            serde_json::to_string_pretty(&config_json)
                .expect("Should serialize config")
                .as_bytes(),
        );

        let fs_manager = FileSystemManager::new(Some(temp_path.to_path_buf()))
            .expect("Should create filesystem manager");
        let mut config_manager = ConfigManager::new(fs_manager);

        let result = config_manager.load_config("config.json").await;
        assert!(
            result.is_ok(),
            "Should load configuration with external files successfully"
        );

        let route = config_manager
            .config
            .routes
            .get("[GET] /api/users")
            .expect("Route should exist");

        match &route.response {
            ConfigResponse::ConfigRouteResponse(response) => {
                assert_eq!(
                    response.status,
                    Some(200),
                    "Status should be loaded from external file"
                );
                match &response.body {
                    BodyResponse::Value(value) => {
                        assert!(
                            value["users"].is_array(),
                            "Should load users array from external file"
                        );
                        assert_eq!(
                            value["users"].as_array().unwrap().len(),
                            2,
                            "Should have 2 users"
                        );
                    }
                    _ => panic!("Body should be JSON value"),
                }
            }
            _ => panic!("Response should be resolved to ConfigRouteResponse"),
        }
    }

    /// Tests configuration loading failure when file doesn't exist.
    ///
    /// Verifies that ConfigManager returns appropriate error when trying
    /// to load a non-existent configuration file.
    #[tokio::test]
    async fn test_config_manager_load_config_file_not_found() {
        let temp_dir = setup_test_dir();
        let fs_manager = FileSystemManager::new(Some(temp_dir.path().to_path_buf()))
            .expect("Should create filesystem manager");
        let mut config_manager = ConfigManager::new(fs_manager);

        let result = config_manager.load_config("nonexistent.json").await;
        assert!(
            result.is_err(),
            "Should fail when configuration file doesn't exist"
        );
    }

    /// Tests configuration loading failure with invalid JSON.
    ///
    /// Verifies that ConfigManager returns appropriate error when trying
    /// to load a configuration file with invalid JSON content.
    #[tokio::test]
    async fn test_config_manager_load_config_invalid_json() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create invalid JSON file
        create_test_file(temp_path, "invalid.json", b"{ invalid json content");

        let fs_manager = FileSystemManager::new(Some(temp_path.to_path_buf()))
            .expect("Should create filesystem manager");
        let mut config_manager = ConfigManager::new(fs_manager);

        let result = config_manager.load_config("invalid.json").await;
        assert!(result.is_err(), "Should fail when JSON content is invalid");
    }

    /// Tests configuration loading failure with empty routes.
    ///
    /// Verifies that ConfigManager returns appropriate error when trying
    /// to load a configuration with no routes defined.
    #[tokio::test]
    async fn test_config_manager_load_config_empty_routes() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create configuration with empty routes
        let config_json = json!({
            "port": 3000,
            "routes": {}
        });

        create_test_file(
            temp_path,
            "config.json",
            serde_json::to_string_pretty(&config_json)
                .expect("Should serialize config")
                .as_bytes(),
        );

        let fs_manager = FileSystemManager::new(Some(temp_path.to_path_buf()))
            .expect("Should create filesystem manager");
        let mut config_manager = ConfigManager::new(fs_manager);

        let result = config_manager.load_config("config.json").await;
        assert!(result.is_err(), "Should fail when routes are empty");

        if let Err(error) = result {
            match error {
                FileSystemError::Operation(msg) => {
                    assert!(
                        msg.contains("routes are empty"),
                        "Error should mention empty routes: {msg}",
                    );
                }
                _ => panic!("Expected Operation error for empty routes"),
            }
        }
    }

    /// Tests configuration saving functionality.
    ///
    /// Verifies that ConfigManager can serialize and save configuration
    /// objects to JSON files.
    #[tokio::test]
    async fn test_config_manager_save_config() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        let fs_manager = FileSystemManager::new(Some(temp_path.to_path_buf()))
            .expect("Should create filesystem manager");
        let config_manager = ConfigManager::new(fs_manager);

        // Create test configuration
        let mut routes = HashMap::new();
        routes.insert(
            "/api/test".to_string(),
            ConfigRoute {
                method: Some("POST".to_string()),
                description: Some("Test endpoint".to_string()),
                headers: None,
                id_field: Some("id".to_string()),
                results_field: None,
                response: ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
                    status: Some(201),
                    body: BodyResponse::Value(json!({"message": "success"})),
                }),
            },
        );

        let config = Config {
            port: Some(9000),
            hostname: Some("example.com".to_string()),
            static_folder: Some("assets".to_string()),
            static_route: "/files".to_string(),
            routes,
        };

        let result = config_manager
            .save_config("test_config.json", &config)
            .await;
        assert!(result.is_ok(), "Should save configuration successfully");

        // Verify the file was created and contains correct data
        let saved_content =
            fs::read_to_string(temp_path.join("test_config.json")).expect("Should read saved file");
        let loaded_config: Config =
            serde_json::from_str(&saved_content).expect("Should parse saved configuration");

        assert_eq!(loaded_config.port, Some(9000), "Port should be preserved");
        assert_eq!(
            loaded_config.hostname,
            Some("example.com".to_string()),
            "Hostname should be preserved"
        );
        assert_eq!(loaded_config.routes.len(), 1, "Routes should be preserved");
    }

    /// Tests configuration file discovery functionality.
    ///
    /// Verifies that ConfigManager can find configuration files using
    /// standard naming conventions.
    #[tokio::test]
    async fn test_config_manager_get_config_file_path() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        let fs_manager = FileSystemManager::new(Some(temp_path.to_path_buf()))
            .expect("Should create filesystem manager");
        let config_manager = ConfigManager::new(fs_manager);

        // Initially no config file should be found
        assert!(
            config_manager.get_config_file_path().is_none(),
            "Should not find config file when none exists"
        );

        // Create db.json file
        create_test_file(temp_path, "db.json", b"{}");
        let found_path = config_manager
            .get_config_file_path()
            .expect("Should find db.json");
        assert!(found_path.ends_with("db.json"), "Should find db.json file");

        // Create .db.json file (should still find db.json first)
        create_test_file(temp_path, ".db.json", b"{}");
        let found_path = config_manager
            .get_config_file_path()
            .expect("Should find config file");
        assert!(
            found_path.ends_with("db.json"),
            "Should prioritize db.json over .db.json"
        );

        // Remove db.json, should find .db.json
        fs::remove_file(temp_path.join("db.json")).expect("Should remove db.json");
        let found_path = config_manager
            .get_config_file_path()
            .expect("Should find .db.json");
        assert!(
            found_path.ends_with(".db.json"),
            "Should find .db.json when db.json is not present"
        );

        // Create json-echo.json (should still find .db.json first)
        create_test_file(temp_path, "json-echo.json", b"{}");
        let found_path = config_manager
            .get_config_file_path()
            .expect("Should find config file");
        assert!(
            found_path.ends_with(".db.json"),
            "Should prioritize .db.json over json-echo.json"
        );

        // Remove .db.json, should find json-echo.json
        fs::remove_file(temp_path.join(".db.json")).expect("Should remove .db.json");
        let found_path = config_manager
            .get_config_file_path()
            .expect("Should find json-echo.json");
        assert!(
            found_path.ends_with("json-echo.json"),
            "Should find json-echo.json when others are not present"
        );
    }

    /// Tests configuration loading with missing external files.
    ///
    /// Verifies that ConfigManager returns appropriate error when external
    /// file references cannot be resolved.
    #[tokio::test]
    async fn test_config_manager_load_config_missing_external_file() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create configuration with missing external file reference
        let config_json = json!({
            "port": 3000,
            "routes": {
                "/api/users": {
                    "method": "GET",
                    "response": "missing_file.json"
                }
            }
        });

        create_test_file(
            temp_path,
            "config.json",
            serde_json::to_string_pretty(&config_json)
                .expect("Should serialize config")
                .as_bytes(),
        );

        let fs_manager = FileSystemManager::new(Some(temp_path.to_path_buf()))
            .expect("Should create filesystem manager");
        let mut config_manager = ConfigManager::new(fs_manager);

        let result = config_manager.load_config("config.json").await;
        assert!(result.is_err(), "Should fail when external file is missing");
    }

    /// Tests configuration round-trip (save and load).
    ///
    /// Verifies that a configuration can be saved and then loaded back
    /// with identical data, ensuring serialization integrity.
    #[allow(clippy::too_many_lines)]
    #[tokio::test]
    async fn test_config_manager_roundtrip() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        let fs_manager = FileSystemManager::new(Some(temp_path.to_path_buf()))
            .expect("Should create filesystem manager");
        let mut config_manager = ConfigManager::new(fs_manager);

        // Create complex configuration
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let mut routes = HashMap::new();
        routes.insert(
            "/api/users".to_string(),
            ConfigRoute {
                method: Some("GET".to_string()),
                description: Some("Get all users".to_string()),
                headers: Some(headers.clone()),
                id_field: Some("user_id".to_string()),
                results_field: Some("data".to_string()),
                response: ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
                    status: Some(200),
                    body: BodyResponse::Value(json!({
                        "users": [
                            {"user_id": 1, "name": "John"},
                            {"user_id": 2, "name": "Jane"}
                        ]
                    })),
                }),
            },
        );

        routes.insert(
            "/api/health".to_string(),
            ConfigRoute {
                method: Some("GET".to_string()),
                description: Some("Health check".to_string()),
                headers: None,
                id_field: Some("id".to_string()),
                results_field: None,
                response: ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
                    status: Some(200),
                    body: BodyResponse::Value(json!({"status": "ok"})),
                }),
            },
        );

        let original_config = Config {
            port: Some(8080),
            hostname: Some("0.0.0.0".to_string()),
            static_folder: Some("public".to_string()),
            static_route: "/assets".to_string(),
            routes,
        };

        // Save configuration
        let save_result = config_manager
            .save_config("roundtrip.json", &original_config)
            .await;
        assert!(
            save_result.is_ok(),
            "Should save configuration successfully"
        );

        // Load configuration back
        let load_result = config_manager.load_config("roundtrip.json").await;
        assert!(
            load_result.is_ok(),
            "Should load configuration successfully"
        );

        // Compare configurations
        let loaded_config = &config_manager.config;
        assert_eq!(
            loaded_config.port, original_config.port,
            "Port should match"
        );
        assert_eq!(
            loaded_config.hostname, original_config.hostname,
            "Hostname should match"
        );
        assert_eq!(
            loaded_config.static_folder, original_config.static_folder,
            "Static folder should match"
        );
        assert_eq!(
            loaded_config.static_route, original_config.static_route,
            "Static route should match"
        );
        assert_eq!(
            loaded_config.routes.len(),
            original_config.routes.len(),
            "Number of routes should match"
        );

        // Verify specific route details
        let users_route = loaded_config
            .routes
            .get("[GET] /api/users")
            .expect("Users route should exist");
        let original_users_route = original_config
            .routes
            .get("/api/users")
            .expect("Original users route should exist");

        assert_eq!(
            users_route.method, original_users_route.method,
            "Route method should match"
        );
        assert_eq!(
            users_route.description, original_users_route.description,
            "Route description should match"
        );
        assert_eq!(
            users_route.id_field, original_users_route.id_field,
            "Route id_field should match"
        );
        assert_eq!(
            users_route.results_field, original_users_route.results_field,
            "Route results_field should match"
        );
    }
}
