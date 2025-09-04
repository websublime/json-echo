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
    extract::{MatchedPath, Path, Query, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
};
use json_echo_core::Database;
use serde_json::{Value, json};
use std::sync::Arc;
use std::{collections::HashMap, io::Error as IOError};
use tower_http::cors::{Any, CorsLayer};
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
///
/// let db = Database::new();
/// let app_state = AppState { db };
/// ```
struct AppState {
    /// The in-memory database containing all route configurations and data
    db: Database,
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
    axum::serve(listener, router).await?;
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
pub fn create_router(db: Database) -> Router {
    info!("Getting models from config");
    // First get the models before moving db
    let models = db.get_models();

    // Create a router with all the routes (no state yet)
    let router_with_routes = models.iter().fold(Router::new(), |router, model| {
        let route_config = db.get_route(model.get_identifier());

        if route_config.is_none() {
            return router;
        }

        let route = route_config.unwrap();

        match route.method.as_deref() {
            Some("GET") => {
                info!("[GET] route defined: {}", model.get_identifier());
                router.route(model.get_identifier(), get(get_handler))
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
    let state = Arc::new(AppState { db });

    // Add CORS and state
    router_with_routes
        .fallback(handler_404)
        .layer(cors)
        .with_state(state)
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
    Path(params): Path<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
    Query(_query_params): Query<HashMap<String, String>>,
    _uri: Uri,
    req: Request,
) -> Response {
    let route_match = req.extensions().get::<MatchedPath>();
    let uri_path = req.uri().path();

    info!("[GET] request called: {uri_path}");

    if route_match.is_none() {
        info!(
            "Route [{uri_path}] not defined with status: {}",
            StatusCode::NOT_FOUND
        );

        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({"error": "Route is not defined"})),
        )
            .into_response();
    }

    let route_path = route_match.unwrap().as_str();
    let model = state.db.get_model(route_path);
    let route = state.db.get_route(route_path);

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
        if !params.is_empty() {
            let model_data = model.find_entry_by_hashmap(params);

            if let Some(data) = model_data {
                return response(headers, StatusCode::OK, &data);
            }
        }

        return response(headers, StatusCode::OK, model.get_data());
    }

    response(
        headers,
        StatusCode::NOT_FOUND,
        &json!({"error": "Model not found"}),
    )
}

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
                let response_data = axum::response::Html(data.to_string());
                debug!("Model Data: {:?}", response_data);
                info!("Response Status: {}", status);
                return (status, headers, response_data).into_response();
            } else if header_type.starts_with("text/plain") {
                let response_data = data.to_string();
                debug!("Model Data: {:?}", response_data);
                info!("Response Status: {}", status);
                return (status, headers, response_data).into_response();
            }
        }
    }

    (status, headers, axum::Json(data)).into_response()
}
