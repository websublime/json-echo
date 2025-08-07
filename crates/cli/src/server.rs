use axum::{
    Router,
    extract::{MatchedPath, Path, Query, Request, State},
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
};
use json_echo_core::Database;
use serde_json::json;
use std::sync::Arc;
use std::{collections::HashMap, io::Error as IOError};
use tower_http::cors::{Any, CorsLayer};

struct AppState {
    db: Database,
}

pub async fn run_server(host: &str, port: &str, router: Router) -> Result<(), IOError> {
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

pub fn create_router(db: Database) -> Router {
    // First get the routes before moving db
    let routes = db.get_routes();

    // Create a router with all the routes (no state yet)
    let router_with_routes = routes.into_iter().fold(Router::new(), |router, route| {
        router.route(route, get(get_handler))
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

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "No route defined")
}

async fn get_handler(
    Path(params): Path<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
    Query(query_params): Query<HashMap<String, String>>,
    uri: Uri,
    req: Request,
) -> Response {
    let route_match = req.extensions().get::<MatchedPath>();

    if route_match.is_none() {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({"error": "Route is not defined"})),
        )
            .into_response();
    }

    let route_path = route_match.unwrap().as_str();
    let model = state.db.get_model(route_path);

    if let Some(model) = model {
        if !params.is_empty() {
            let model_data = model.find_entry_by_hashmap(params);

            if let Some(data) = model_data {
                return (StatusCode::OK, axum::Json(data)).into_response();
            }
        }
        return (StatusCode::OK, axum::Json(model.get_data())).into_response();
    }

    (
        StatusCode::NOT_FOUND,
        axum::Json(json!({"error": "Model not found"})),
    )
        .into_response()
}
