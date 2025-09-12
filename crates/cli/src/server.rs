//! HTTP server implementation for the JSON Echo application.
//!
//! This module provides the web server functionality for JSON Echo, including
//! router creation, request handling, and HTTP response generation. It uses
//! the Axum web framework to serve mock API responses based on the configured
//! routes and data models.
//!
//! ## What
//!
//! The module defines:
//! - `AppState`: Application state container holding the database
//! - `run_server`: Function to start the HTTP server
//! - `create_router`: Function to build the Axum router with all routes
//! - Request handlers for different HTTP scenarios (GET, 404)
//!
//! ## How
//!
//! The server works by:
//! 1. Creating an Axum router with dynamically generated routes from the database
//! 2. Setting up CORS middleware for cross-origin requests
//! 3. Configuring request handlers that query the in-memory database
//! 4. Serving responses based on route parameters and query strings
//! 5. Providing fallback handling for undefined routes
//!
//! ## Why
//!
//! This design enables:
//! - Dynamic route generation based on configuration
//! - Flexible request parameter handling (path and query parameters)
//! - CORS support for web application integration
//! - Type-safe request and response handling
//! - Scalable architecture with shared application state
//!
//! # Examples
//!
//! ```rust
//! use json_echo_core::Database;
//! // This would typically be called from main.rs
//! // let router = create_router(database);
//! // run_server("localhost", "3000", router).await?;
//! ```

use axum::{
    Router,
    extract::{Json, MatchedPath, Path, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use json_echo_core::{ConfigManager, Database};
use serde_json::{Value, json};
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, io::Error as IOError};
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{debug, info};

/// Application state container that holds shared data across request handlers.
///
/// `AppState` encapsulates the application's shared state, primarily the
/// in-memory database containing route configurations and mock data. This
/// state is shared across all request handlers through Axum's state system.
///
/// # Fields
///
/// * `db` - The in-memory database containing route definitions and mock data
///
/// # Examples
///
/// ```rust
/// use json_echo_core::Database;
/// use std::sync::RwLock;
///
/// let db = Database::new();
/// let app_state = AppState { db: RwLock::new(db) };
/// ```
struct AppState {
    /// The in-memory database containing all route configurations and data
    db: RwLock<Database>,
}

/// Starts the HTTP server on the specified host and port with the given router.
///
/// This asynchronous function creates a TCP listener and starts serving HTTP
/// requests using the provided Axum router. It handles the low-level server
/// setup and request dispatching.
///
/// # Parameters
///
/// * `host` - The hostname or IP address to bind the server to
/// * `port` - The port number to listen on
/// * `router` - The configured Axum router with all routes and middleware
///
/// # Returns
///
/// * `Ok(())` - If the server started and ran successfully
/// * `Err(IOError)` - If the server failed to bind or encountered network errors
///
/// # Errors
///
/// This function can fail if:
/// - The specified host/port combination is already in use
/// - The host address is invalid or unreachable
/// - Network permissions prevent binding to the specified port
/// - System resource limits are exceeded
///
/// # Examples
///
/// ```rust
/// use axum::Router;
/// use std::io::Error;
///
/// # async fn example() -> Result<(), Error> {
/// let router = Router::new();
/// run_server("localhost", "3000", router).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_server(host: &str, port: &str, router: Router) -> Result<(), IOError> {
    info!("Starting server at: http://{}:{}", host, port);

    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Creates and configures the Axum router with all routes and middleware.
///
/// This function builds the complete router configuration by iterating through
/// the database models and creating corresponding HTTP routes. It sets up
/// CORS middleware, adds request handlers, and configures fallback behavior
/// for undefined routes.
///
/// # Parameters
///
/// * `db` - The database containing route configurations and mock data
///
/// # Returns
///
/// A fully configured `Router` ready to handle HTTP requests
///
/// # Behavior
///
/// The function performs the following setup:
/// 1. Iterates through all models in the database
/// 2. Creates HTTP routes based on model identifiers and methods
/// 3. Currently supports GET requests (extensible for other methods)
/// 4. Configures CORS middleware for cross-origin requests
/// 5. Sets up a 404 fallback handler for undefined routes
/// 6. Wraps the database in shared application state
///
/// # CORS Configuration
///
/// The router includes permissive CORS settings:
/// - Allows all HTTP methods (GET, POST, PUT, PATCH, DELETE, OPTIONS)
/// - Allows all headers and origins
/// - Disables credentials for security
///
/// # Examples
///
/// ```rust
/// use json_echo_core::Database;
///
/// let mut db = Database::new();
/// // db would be populated with route configurations
/// let router = create_router(db);
/// // Router is now ready to handle requests
/// ```
pub fn create_router(db: Database, config_manager: &ConfigManager) -> Router {
    info!("Getting models from config");
    // First get the routes before moving db
    let routes = db.get_routes();
    let config = &config_manager.config;

    // Create a router with all the routes (no state yet)
    let router_with_routes = routes.iter().fold(Router::new(), |router, route| {
        let route_config = db.get_route(route, None);

        if route_config.is_none() {
            info!("⚠︎ Route {} as no configuration associated", route);
            return router;
        }

        let route_method = route_config.unwrap().method.as_deref();
        let route_path = extract_path(route);

        match route_method {
            Some("GET") => {
                info!("[GET] route defined: {}", route_path);
                router.route(route_path, get(get_handler))
            }
            Some("POST") => {
                info!("[POST] route defined: {}", route_path);
                router.route(route_path, post(post_handler))
            }
            Some("PUT") => {
                info!("[PUT] route defined: {}", route_path);
                router.route(route_path, put(put_handler))
            }
            _ => router,
        }
    });

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .allow_origin(Any)
        .allow_credentials(false);

    // Move db into state after we're done using routes
    let state = Arc::new(AppState {
        db: RwLock::new(db),
    });

    // Add CORS and state
    let router = router_with_routes
        .fallback(handler_404)
        .layer(cors)
        .with_state(state);

    if let Some(static_folder) = config.static_folder.as_ref() {
        let static_route = config.static_route.as_str();

        info!(
            "Serving static files from: {}, on route {}",
            static_folder, static_route
        );
        let serve_dir = ServeDir::new(config_manager.get_root().join(static_folder));
        return router.nest_service(static_route, serve_dir);
    }

    router
}

/// Fallback handler for undefined routes (404 Not Found).
///
/// This handler is called when a request is made to a route that is not
/// defined in the router configuration. It returns a simple 404 status
/// with a descriptive message.
///
/// # Returns
///
/// An HTTP response with status 404 and explanatory text
///
/// # Examples
///
/// When a client requests a non-existent route like `/undefined-path`,
/// this handler will return:
/// ```
/// HTTP/1.1 404 Not Found
/// Content-Type: text/plain
///
/// No route defined
/// ```
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "No route defined")
}

/// HTTP GET request handler that serves mock data based on route configuration.
///
/// This handler processes GET requests by looking up the corresponding model
/// in the database and returning appropriate mock data. It supports both
/// parameterized requests (returning specific entries) and general requests
/// (returning all data).
///
/// # Parameters
///
/// * `Path(params)` - Path parameters extracted from the URL
/// * `State(state)` - Shared application state containing the database
/// * `Query(query_params)` - Query string parameters from the request
/// * `uri` - The full URI of the request
/// * `req` - The complete HTTP request object
///
/// # Returns
///
/// An HTTP response containing:
/// - JSON data from the model if found
/// - Specific entry if path parameters match
/// - All model data if no specific parameters
/// - 404 error if route or model not found
///
/// # Behavior
///
/// The handler follows this logic:
/// 1. Extracts the matched route path from request extensions
/// 2. Looks up the corresponding model in the database
/// 3. If path parameters are provided, searches for a specific entry
/// 4. Returns the specific entry if found, or all model data otherwise
/// 5. Returns appropriate error responses for missing routes/models
///
/// # Response Format
///
/// Successful responses return JSON data with status 200:
/// ```json
/// {
///   "id": 1,
///   "name": "example",
///   "data": "value"
/// }
/// ```
///
/// Error responses return JSON with appropriate status codes:
/// ```json
/// {
///   "error": "Route is not defined"
/// }
/// ```
///
/// # Examples
///
/// ```
/// GET /users -> Returns all users
/// GET /users/123 -> Returns user with ID 123 (if found)
/// GET /undefined -> Returns 404 error
/// ```
async fn get_handler(
    State(state): State<Arc<AppState>>,
    Path(params): Path<HashMap<String, String>>,
    uri_path: Uri,
    path: MatchedPath,
) -> Response {
    info!("[GET] request called: {}", uri_path.path());
    let state_reader = state.db.read();

    if state_reader.is_err() {
        return response(
            HeaderMap::new(),
            StatusCode::EXPECTATION_FAILED,
            &json!({"error": "Unable to read database"}),
        );
    }

    let state_reader = state_reader.unwrap();

    let route_path = path.as_str();
    let model = state_reader.get_model(&format!("[GET] {route_path}"));
    let route = state_reader.get_route(route_path, Some(String::from("GET")));

    debug!("Model: {:?}", model);
    debug!("Route Config: {:?}", route);

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    if let Some(route) = route {
        if let Some(route_headers) = &route.headers {
            for (key, value) in route_headers {
                if let Ok(header_name) = key.parse::<HeaderName>() {
                    if let Ok(header_value) = value.parse() {
                        headers.insert(header_name, header_value);
                    }
                }
            }
        }
    }

    debug!("Headers Config: {:?}", headers);

    if let Some(model) = model {
        let http_status = model.get_status().unwrap_or(StatusCode::OK.as_u16());
        let status = StatusCode::from_u16(http_status).unwrap_or(StatusCode::OK);

        if !params.is_empty() {
            let model_data = model.find_entry_by_hashmap(params);

            if let Some(data) = model_data {
                return response(headers, status, &data);
            }
        }

        let response_body = model.get_data();

        return response(headers, status, &response_body.as_value());
    }

    response(
        headers,
        StatusCode::NOT_FOUND,
        &json!({"error": "Model not found"}),
    )
}

/// HTTP POST request handler that processes incoming data and serves mock responses.
///
/// This handler processes POST requests by accepting JSON payloads and returning
/// appropriate mock data based on route configuration. It supports payload
/// validation and dynamic response generation.
///
/// # Parameters
///
/// * `Path(params)` - Path parameters extracted from the URL
/// * `State(state)` - Shared application state containing the database
/// * `payload` - Optional JSON payload from the request body
/// * `req` - The complete HTTP request object
///
/// # Returns
///
/// An HTTP response containing:
/// - JSON data from the model if found
/// - Confirmation of data processing if successful
/// - 404 error if route or model not found
///
/// # Behavior
///
/// The handler follows this logic:
/// 1. Extracts the matched route path from request extensions
/// 2. Looks up the corresponding model in the database
/// 3. Processes the incoming JSON payload if provided
/// 4. Returns appropriate mock response based on configuration
/// 5. Returns error responses for missing routes/models
///
/// # Examples
///
/// ```
/// POST /users -> Creates/processes user data
/// POST /api/data -> Processes API data submission
/// ```
#[allow(clippy::redundant_else)]
async fn post_handler(
    State(state): State<Arc<AppState>>,
    Path(params): Path<HashMap<String, String>>,
    uri_path: Uri,
    path: MatchedPath,
    payload: Option<Json<Value>>,
) -> Response {
    info!("[POST] request called: {}", uri_path.path());

    let body_payload = payload.unwrap_or(axum::Json(json!({})));
    let route_path = path.as_str();
    let route_identifier = format!("[POST] {route_path}");

    // First, get the route configuration and model info without holding the lock
    let (model_exists, route_headers, model_status) = {
        let state_reader = match state.db.read() {
            Ok(reader) => reader,
            Err(_) => {
                return response(
                    HeaderMap::new(),
                    StatusCode::EXPECTATION_FAILED,
                    &json!({"error": "Unable to read database"}),
                );
            }
        };

        let model = state_reader
            .get_model(&format!("[GET] {route_path}"))
            .or_else(|| state_reader.get_model(&format!("[POST] {route_path}")));
        let route_config = state_reader.get_route(route_path, Some(String::from("GET")));

        debug!("Route Config: {:?}", route_config);
        debug!("Payload: {:?}", body_payload);

        let model_exists = model.is_some();
        let route_headers = route_config.and_then(|rc| rc.headers.clone());
        let model_status = model.map(|m| m.get_status().unwrap_or(StatusCode::OK.as_u16()));

        (model_exists, route_headers, model_status)
    }; // Read lock liberado aqui

    if !model_exists {
        return response(
            HeaderMap::new(),
            StatusCode::NOT_FOUND,
            &json!({"error": "Model not found"}),
        );
    }

    // Configurar headers
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    if let Some(route_headers) = route_headers {
        for (key, value) in route_headers {
            if let Ok(header_name) = key.parse::<HeaderName>() {
                if let Ok(header_value) = value.parse() {
                    headers.insert(header_name, header_value);
                }
            }
        }
    }

    debug!("Headers Config: {:?}", headers);

    let http_status = model_status.unwrap_or(StatusCode::OK.as_u16());
    let status = StatusCode::from_u16(http_status).unwrap_or(StatusCode::OK);
    let payload_data = body_payload.0;

    // Phase 2: Update data (write lock)
    {
        let mut state_writer = match state.db.write() {
            Ok(writer) => writer,
            Err(_) => {
                return response(
                    HeaderMap::new(),
                    StatusCode::EXPECTATION_FAILED,
                    &json!({"error": "Unable to write to database"}),
                );
            }
        };

        match state_writer.update_model_data(&route_identifier, payload_data.clone()) {
            Ok(_) => {
                info!("✔︎ Model data updated: {route_identifier}");

                // Sync with GET model
                let get_identifier = format!("[GET] {route_path}");
                if let Ok(_) = state_writer.update_model_data(&get_identifier, payload_data) {
                    info!("✔︎ GET Model data updated: {get_identifier}");
                }
            }
            Err(e) => {
                info!("⚠︎ Failed to update model data: {route_identifier}");
                debug!("Update model error: {:?}", e);
            }
        }
    } // Write lock droped

    // Phase 3: Get response data (new read lock)
    let state_reader = match state.db.read() {
        Ok(reader) => reader,
        Err(_) => {
            return response(
                headers,
                StatusCode::INTERNAL_SERVER_ERROR,
                &json!({"error": "Unable to read updated data"}),
            );
        }
    };

    if let Some(model) = state_reader.get_model(&route_identifier) {
        if !params.is_empty() {
            if let Some(data) = model.find_entry_by_hashmap(params) {
                return response(headers, status, &data);
            }
        }

        let response_body = model.get_data();
        return response(headers, status, &response_body.as_value());
    }

    response(
        headers,
        StatusCode::NOT_FOUND,
        &json!({"error": "Model not found"}),
    )
}

async fn put_handler(
    State(_state): State<Arc<AppState>>,
    Path(_params): Path<HashMap<String, String>>,
    uri_path: Uri,
    _path: MatchedPath,
    _payload: Option<Json<Value>>,
) -> Response {
    info!("[PUT] request called: {}", uri_path.path());

    response(
        HeaderMap::new(),
        StatusCode::NOT_FOUND,
        &json!({"error": "Model not found"}),
    )
}
/// Creates an HTTP response with the appropriate content type and format.
///
/// This function generates HTTP responses by examining the provided headers
/// to determine the appropriate response format (JSON, Form, HTML, or plain text).
/// It handles different content types and serializes the data accordingly.
///
/// # Parameters
///
/// * `headers` - HTTP headers that will be included in the response
/// * `status` - HTTP status code for the response
/// * `data` - JSON value containing the response data to be serialized
///
/// # Returns
///
/// A configured `Response` object ready to be sent to the client
///
/// # Behavior
///
/// The function examines the `content-type` header to determine output format:
/// - `application/x-www-form-urlencoded` - Returns form-encoded data
/// - `text/html` - Returns HTML content (extracts string from JSON)
/// - `text/plain` - Returns plain text (extracts string from JSON)
/// - Default - Returns JSON-encoded data
///
/// # Content Type Handling
///
/// - **Form Data**: Wraps the JSON value in `axum::extract::Form`
/// - **HTML**: Extracts string content and wraps in `axum::response::Html`
/// - **Plain Text**: Extracts string content and returns as plain text
/// - **JSON**: Default format using `axum::Json` wrapper
///
/// # Examples
///
/// ```rust
/// use axum::http::{HeaderMap, StatusCode};
/// use serde_json::json;
///
/// let headers = HeaderMap::new();
/// let data = json!({"message": "Hello World"});
/// let response = response(headers, StatusCode::OK, &data);
/// // Returns JSON response with status 200
/// ```
///
/// For HTML responses:
/// ```rust
/// let mut headers = HeaderMap::new();
/// headers.insert("content-type", "text/html".parse().unwrap());
/// let data = json!("<h1>Hello World</h1>");
/// let response = response(headers, StatusCode::OK, &data);
/// // Returns HTML response
/// ```
fn response(headers: HeaderMap, status: StatusCode, data: &Value) -> Response {
    // Check header content type and use axum (Json, Form or simple text)
    if let Some(content_type) = headers.get("content-type") {
        if let Ok(header_type) = content_type.to_str() {
            if header_type.starts_with("application/x-www-form-urlencoded") {
                let response_data = axum::extract::Form(data.clone());
                debug!("Model Data: {:?}", response_data);
                info!("Response Status: {}", status);
                return (status, headers, response_data).into_response();
            } else if header_type.starts_with("text/html") {
                let string_data = match data {
                    Value::String(value) => value,
                    _ => "",
                };

                let response_data = axum::response::Html(string_data.to_string());
                debug!("Model Data: {:?}", response_data);
                info!("Response Status: {}", status);
                return (status, headers, response_data).into_response();
            } else if header_type.starts_with("text/plain") {
                let response_data = match data {
                    Value::String(value) => value,
                    _ => "",
                };

                debug!("Model Data: {:?}", response_data.to_string());
                info!("Response Status: {}", status);
                return (status, headers, response_data.to_string()).into_response();
            }
        }
    }

    (status, headers, axum::Json(data)).into_response()
}

/// Extracts the URL path from a route pattern string.
///
/// This function parses route pattern strings that may contain HTTP method
/// prefixes in square brackets and returns the clean URL path portion.
/// It handles both bracketed patterns and plain path strings.
///
/// # Parameters
///
/// * `pattern` - The route pattern string to parse, may include method prefix
///
/// # Returns
///
/// A string slice containing the extracted URL path
///
/// # Pattern Format
///
/// The function expects patterns in one of these formats:
/// - `[METHOD] /path/to/endpoint` - Method prefix with path
/// - `/path/to/endpoint` - Plain path without method prefix
///
/// # Behavior
///
/// The function:
/// 1. Searches for the closing bracket `]` in the pattern
/// 2. If found, extracts everything after the bracket and trims whitespace
/// 3. If no bracket found, returns the original pattern unchanged
///
/// # Examples
///
/// ```rust
/// assert_eq!(extract_path("[GET] /users"), "/users");
/// assert_eq!(extract_path("[POST] /api/data"), "/api/data");
/// assert_eq!(extract_path("/simple/path"), "/simple/path");
/// assert_eq!(extract_path("[DELETE] /users/:id"), "/users/:id");
/// ```
///
/// # Use Cases
///
/// This function is typically used when:
/// - Processing route configurations that include HTTP methods
/// - Converting internal route representations to Axum-compatible paths
/// - Cleaning route patterns for router registration
fn extract_path(pattern: &str) -> &str {
    if let Some(end) = pattern.find(']') {
        let path = &pattern[end + 1..].trim();
        return path;
    }
    pattern
}

#[allow(clippy::ignored_unit_patterns)]
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
