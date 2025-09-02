# JSON Echo Core

A robust Rust library for creating mock API servers with JSON-based configuration and dynamic route handling.

## Overview

JSON Echo Core provides the foundational components for building mock API servers that can serve predefined JSON responses based on flexible configuration files. It's designed for testing, prototyping, and development scenarios where you need controllable API responses without implementing full backend services.

## Features

- **Configuration Management**: Load and manage JSON-based configuration files with automatic validation
- **Dynamic Route Handling**: Support for flexible route definitions with parameterized responses
- **In-Memory Database**: Fast, queryable data storage for mock responses
- **Filesystem Abstraction**: Cross-platform file operations with proper error handling
- **Type-Safe Error Handling**: Comprehensive error types for robust error propagation
- **Async Support**: Full async/await support for non-blocking operations

## Core Components

### Configuration System

The configuration system (`config` module) handles loading, parsing, and managing JSON configuration files:

```rust
use json_echo_core::{ConfigManager, FileSystemManager};

let fs_manager = FileSystemManager::new(None)?;
let mut config_manager = ConfigManager::new(fs_manager);
config_manager.load_config("api-config.json").await?;
```

### Database Management

The database system (`database` module) provides in-memory storage and querying:

```rust
use json_echo_core::Database;

let mut db = Database::new();
db.populate(config_manager.config.routes);

// Query for specific models
if let Some(model) = db.get_model("users") {
    let data = model.get_data();
    println!("Model data: {}", data);
}
```

### Filesystem Operations

The filesystem module (`filesystem` module) offers utilities for file operations and project discovery:

```rust
use json_echo_core::{FileSystemManager, PathUtils};

// Automatic project root discovery
let fs_manager = FileSystemManager::new(None)?;

// Load configuration data
let config_data = fs_manager.load_file("config.json").await?;
```

### Error Handling

Comprehensive error handling with specific error types for different failure scenarios:

```rust
use json_echo_core::{FileSystemError, FileSystemResult};

fn load_data() -> FileSystemResult<String> {
    // Operations that might fail with filesystem errors
    Ok("data".to_string())
}

match load_data() {
    Ok(data) => println!("Loaded: {}", data),
    Err(FileSystemError::NotFound { path }) => {
        eprintln!("File not found: {}", path.display());
    },
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Configuration Format

JSON Echo uses JSON configuration files to define server behavior:

```json
{
  "port": 3001,
  "hostname": "localhost",
  "routes": {
    "/api/users": {
      "method": "GET",
      "description": "Get all users",
      "id_field": "id",
      "response": {
        "status": 200,
        "body": [
          {"id": 1, "name": "John Doe", "email": "john@example.com"},
          {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
        ]
      }
    },
    "/api/users/:id": {
      "method": "GET",
      "description": "Get user by ID",
      "id_field": "id",
      "response": {
        "status": 200,
        "body": {"id": 1, "name": "John Doe", "email": "john@example.com"}
      }
    }
  }
}
```

## Usage Examples

### Basic Setup

```rust
use json_echo_core::{
    ConfigManager, Database, FileSystemManager, 
    Config, ConfigRoute, ConfigResponse, ConfigRouteResponse
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize filesystem manager
    let fs_manager = FileSystemManager::new(None)?;
    let mut config_manager = ConfigManager::new(fs_manager);
    
    // Load configuration
    config_manager.load_config("server-config.json").await?;
    
    // Setup database
    let mut db = Database::new();
    db.populate(config_manager.config.routes.clone());
    
    // Query data
    let models = db.get_models();
    for model in models {
        println!("Route: {}", model.get_identifier());
        if let Some(desc) = model.get_description() {
            println!("  Description: {}", desc);
        }
    }
    
    Ok(())
}
```

### Creating Configuration Programmatically

```rust
use json_echo_core::{Config, ConfigRoute, ConfigResponse, ConfigRouteResponse};
use serde_json::json;
use std::collections::HashMap;

let mut routes = HashMap::new();

routes.insert("/api/health".to_string(), ConfigRoute {
    method: Some("GET".to_string()),
    description: Some("Health check endpoint".to_string()),
    headers: None,
    id_field: None,
    results_field: None,
    response: ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
        status: Some(200),
        body: json!({"status": "ok", "timestamp": "2024-01-01T00:00:00Z"}),
    }),
});

let config = Config {
    port: Some(3000),
    hostname: Some("0.0.0.0".to_string()),
    routes,
};
```

### File Operations

```rust
use json_echo_core::FileSystemManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fs_manager = FileSystemManager::new(None)?;
    
    // Load JSON data
    let data = fs_manager.load_file("data.json").await?;
    let json_text = String::from_utf8(data)?;
    let parsed: serde_json::Value = serde_json::from_str(&json_text)?;
    
    // Save processed data
    let output = serde_json::to_vec_pretty(&parsed)?;
    fs_manager.save_file("output.json", output).await?;
    
    Ok(())
}
```

## Error Handling

The library provides comprehensive error handling with specific error types:

- `FileSystemError`: File and directory operation errors
- `Error`: General application errors
- `FileSystemResult<T>`: Type alias for Results with filesystem errors

All errors implement standard traits and provide detailed context information.

## Async Support

All I/O operations are fully asynchronous and use Tokio for the async runtime:

```rust
// Async file loading
let config_data = config_manager.load_config("config.json").await?;

// Async file saving
config_manager.save_config("backup.json", &config).await?;
```

## Project Structure

The library is organized into focused modules:

- `config`: Configuration file management and parsing
- `database`: In-memory data storage and querying
- `filesystem`: File system operations and utilities
- `errors`: Error types and result handling

## Minimum Supported Rust Version (MSRV)

This crate requires Rust 1.70.0 or later.

## License

This project is licensed under the MIT License - see the LICENSE file for details.