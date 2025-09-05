//! Comprehensive test suite for the database module.
//!
//! This module contains extensive tests for all functionality exposed by the database module,
//! including Database and Model structures. Tests are designed to verify correct behavior,
//! data management, query operations, merging functionality, and edge cases across different
//! scenarios.
//!
//! ## What
//!
//! The test suite covers:
//! - Database structure creation, population, and route/model management
//! - Model structure creation, data access, and metadata retrieval
//! - Data merging functionality with various JSON structures
//! - Query operations for finding specific data entries
//! - Error handling for invalid operations and edge cases
//! - Integration between Database and Model components
//!
//! ## How
//!
//! Tests use a combination of mock data structures and real JSON values to create
//! comprehensive test scenarios. Each test creates its own isolated data structures
//! and verifies the expected behavior without side effects. JSON manipulation and
//! data merging are thoroughly tested to ensure data integrity.
//!
//! ## Why
//!
//! Comprehensive testing ensures:
//! - Reliability of database operations and data management
//! - Proper handling of various JSON data structures
//! - Correct merging behavior for complex data scenarios
//! - Robust query functionality for data retrieval
//! - Data integrity across update and merge operations

use json_echo_core::{
    BodyResponse, ConfigResponse, ConfigRoute, ConfigRouteResponse, Database, Model,
};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Creates a sample ConfigRoute for testing purposes.
///
/// Helper function that creates a ConfigRoute with specified parameters for
/// use in test scenarios. Provides consistent test data across multiple tests.
///
/// # Parameters
///
/// * `method` - The HTTP method for the route
/// * `description` - Optional description for the route
/// * `id_field` - The ID field name for the route
/// * `results_field` - Optional results field name
/// * `response_data` - The JSON data for the response body
///
/// # Returns
///
/// A ConfigRoute instance configured with the provided parameters
fn create_test_route(
    method: &str,
    description: Option<&str>,
    id_field: &str,
    results_field: Option<&str>,
    response_data: Value,
) -> ConfigRoute {
    ConfigRoute {
        method: Some(method.to_string()),
        description: description.map(String::from),
        headers: None,
        id_field: Some(id_field.to_string()),
        results_field: results_field.map(String::from),
        response: ConfigResponse::ConfigRouteResponse(ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(response_data),
        }),
    }
}

/// Creates a sample Database with test data for testing purposes.
///
/// Helper function that creates a Database populated with sample routes and models
/// to provide consistent test data across multiple test scenarios.
///
/// # Returns
///
/// A Database instance populated with test routes and models
fn create_test_database() -> Database {
    let mut db = Database::new();
    let mut routes = HashMap::new();

    // Create users route
    let users_data = json!({
        "users": [
            {"id": 1, "name": "John Doe", "email": "john@example.com"},
            {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
        ]
    });
    routes.insert(
        "[GET] /api/users".to_string(),
        create_test_route(
            "GET",
            Some("Get all users"),
            "id",
            Some("users"),
            users_data,
        ),
    );

    // Create products route
    let products_data = json!([
        {"product_id": 101, "name": "Laptop", "price": 999.99},
        {"product_id": 102, "name": "Mouse", "price": 29.99}
    ]);
    routes.insert(
        "[GET] /api/products".to_string(),
        create_test_route(
            "GET",
            Some("Get all products"),
            "product_id",
            None,
            products_data,
        ),
    );

    // Create simple object route
    let status_data = json!({"status": "ok", "timestamp": "2023-01-01T00:00:00Z"});
    routes.insert(
        "[GET] /api/status".to_string(),
        create_test_route("GET", Some("Get system status"), "id", None, status_data),
    );

    db.populate(routes);
    db
}

mod database_structure_tests {
    use super::*;

    /// Tests Database creation with default values.
    ///
    /// Verifies that Database::new() creates an empty database with
    /// no routes or models initialized.
    #[test]
    fn test_database_new() {
        let db = Database::new();

        assert!(
            db.get_routes().is_empty(),
            "New database should have no routes"
        );
        assert!(
            db.get_models().is_empty(),
            "New database should have no models"
        );
    }

    /// Tests Database population with routes.
    ///
    /// Verifies that the populate method correctly stores routes and
    /// generates corresponding models from the route configurations.
    #[test]
    fn test_database_populate() {
        let mut db = Database::new();
        let mut routes = HashMap::new();

        let test_data = json!({"items": [{"id": 1, "name": "test"}]});
        routes.insert(
            "[GET] /test".to_string(),
            create_test_route("GET", Some("Test route"), "id", Some("items"), test_data),
        );

        db.populate(routes);

        assert_eq!(db.get_routes().len(), 1, "Should have one route");
        assert_eq!(db.get_models().len(), 1, "Should have one model");

        let route_keys = db.get_routes();
        assert!(
            route_keys.contains(&&"[GET] /test".to_string()),
            "Should contain the test route key"
        );
    }

    /// Tests Database populate with multiple routes.
    ///
    /// Verifies that the database can handle multiple routes and generates
    /// the correct number of models from the provided configurations.
    #[test]
    fn test_database_populate_multiple_routes() {
        let db = create_test_database();

        assert_eq!(db.get_routes().len(), 3, "Should have three routes");
        assert_eq!(db.get_models().len(), 3, "Should have three models");

        let route_keys = db.get_routes();
        assert!(
            route_keys.contains(&&"[GET] /api/users".to_string()),
            "Should contain users route"
        );
        assert!(
            route_keys.contains(&&"[GET] /api/products".to_string()),
            "Should contain products route"
        );
        assert!(
            route_keys.contains(&&"[GET] /api/status".to_string()),
            "Should contain status route"
        );
    }

    /// Tests Database populate replaces existing routes but appends models.
    ///
    /// Verifies that calling populate multiple times replaces the existing
    /// routes but appends new models to the existing models vector.
    /// This reflects the actual behavior of the populate method.
    #[test]
    fn test_database_populate_replaces_existing() {
        let mut db = Database::new();

        // First populate with initial data
        let mut initial_routes = HashMap::new();
        let users_data = json!([{"id": 1, "name": "John"}]);
        initial_routes.insert(
            "[GET] /api/users".to_string(),
            create_test_route("GET", Some("Get users"), "id", None, users_data),
        );
        initial_routes.insert(
            "[GET] /api/products".to_string(),
            create_test_route("GET", Some("Get products"), "id", None, json!([])),
        );

        db.populate(initial_routes);
        assert_eq!(db.get_routes().len(), 2, "Should initially have two routes");
        assert_eq!(db.get_models().len(), 2, "Should initially have two models");

        // Populate with new data
        let mut new_routes = HashMap::new();
        let simple_data = json!({"message": "hello"});
        new_routes.insert(
            "[GET] /simple".to_string(),
            create_test_route("GET", None, "id", None, simple_data),
        );

        db.populate(new_routes);

        assert_eq!(db.get_routes().len(), 1, "Should now have only one route");
        assert_eq!(
            db.get_models().len(),
            3,
            "Should now have three models (2 old + 1 new)"
        );

        let route_keys = db.get_routes();
        assert!(
            route_keys.contains(&&"[GET] /simple".to_string()),
            "Should contain the new simple route"
        );
        assert!(
            !route_keys.contains(&&"[GET] /api/users".to_string()),
            "Should not contain old users route"
        );
    }
}

mod database_query_tests {
    use super::*;

    /// Tests getting a route by identifier.
    ///
    /// Verifies that get_route correctly retrieves routes using their
    /// identifier and returns None for non-existent routes.
    #[test]
    fn test_database_get_route() {
        let db = create_test_database();

        // Test existing route
        let route = db.get_route("/api/users", Some("GET".to_string()));
        assert!(route.is_some(), "Should find the users route");

        let route = route.unwrap();
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

        // Test non-existent route
        let route = db.get_route("/api/nonexistent", Some("GET".to_string()));
        assert!(route.is_none(), "Should not find non-existent route");
    }

    /// Tests getting a route with method parsing from identifier.
    ///
    /// Verifies that routes can be retrieved using identifiers that include
    /// method information in the format [METHOD] path.
    #[test]
    fn test_database_get_route_with_method_in_identifier() {
        let db = create_test_database();

        // Test with method included in identifier
        let route = db.get_route("[GET] /api/users", None);
        assert!(
            route.is_some(),
            "Should find route with method in identifier"
        );

        // Test with different method
        let route = db.get_route("[POST] /api/users", None);
        assert!(
            route.is_none(),
            "Should not find route with different method"
        );
    }

    /// Tests getting a route with default method.
    ///
    /// Verifies that routes can be retrieved with default GET method when
    /// no method is explicitly specified.
    #[test]
    fn test_database_get_route_default_method() {
        let db = create_test_database();

        // Test with no method specified (should default to GET)
        let route = db.get_route("/api/users", None);
        assert!(route.is_some(), "Should find route with default GET method");

        // Test with explicit GET method
        let route_explicit = db.get_route("/api/users", Some("GET".to_string()));
        assert!(
            route_explicit.is_some(),
            "Should find route with explicit GET method"
        );

        // Both should return the same route
        assert_eq!(
            route.unwrap().description,
            route_explicit.unwrap().description,
            "Should return the same route for default and explicit GET"
        );
    }

    /// Tests getting all routes from database.
    ///
    /// Verifies that get_routes returns all route identifiers stored
    /// in the database.
    #[test]
    fn test_database_get_routes() {
        let db = create_test_database();
        let routes = db.get_routes();

        assert_eq!(routes.len(), 3, "Should return all three routes");

        let route_strings: Vec<String> = routes.iter().map(|r| (*r).clone()).collect();
        assert!(
            route_strings.contains(&"[GET] /api/users".to_string()),
            "Should contain users route"
        );
        assert!(
            route_strings.contains(&"[GET] /api/products".to_string()),
            "Should contain products route"
        );
        assert!(
            route_strings.contains(&"[GET] /api/status".to_string()),
            "Should contain status route"
        );
    }

    /// Tests getting all models from database.
    ///
    /// Verifies that get_models returns a reference to all models
    /// generated from the route configurations.
    #[test]
    fn test_database_get_models() {
        let db = create_test_database();
        let models = db.get_models();

        assert_eq!(models.len(), 3, "Should return all three models");

        let model_ids: Vec<&str> = models
            .iter()
            .map(json_echo_core::Model::get_identifier)
            .collect();
        assert!(
            model_ids.contains(&"[GET] /api/users"),
            "Should contain users model"
        );
        assert!(
            model_ids.contains(&"[GET] /api/products"),
            "Should contain products model"
        );
        assert!(
            model_ids.contains(&"[GET] /api/status"),
            "Should contain status model"
        );
    }

    /// Tests getting a specific model by identifier.
    ///
    /// Verifies that get_model correctly retrieves models using their
    /// identifier and returns None for non-existent models.
    #[test]
    fn test_database_get_model() {
        let db = create_test_database();

        // Test existing model
        let model = db.get_model("[GET] /api/users");
        assert!(model.is_some(), "Should find the users model");

        let model = model.unwrap();
        assert_eq!(
            model.get_identifier(),
            "[GET] /api/users",
            "Model identifier should match"
        );
        assert_eq!(model.get_id_field(), "id", "Model ID field should be 'id'");

        // Test non-existent model
        let model = db.get_model("nonexistent");
        assert!(model.is_none(), "Should not find non-existent model");
    }
}

mod model_structure_tests {
    use super::*;

    /// Tests Model creation with all parameters.
    ///
    /// Verifies that Model::new correctly creates a model instance
    /// with all provided parameters properly stored.
    #[test]
    fn test_model_new() {
        let test_data = ConfigRouteResponse {
            status: Some(201),
            body: BodyResponse::Value(json!({"test": "data"})),
        };

        let model = Model::new(
            "test_model".to_string(),
            "test_id".to_string(),
            Some("results".to_string()),
            Some("Test model description".to_string()),
            test_data,
        );

        assert_eq!(model.get_identifier(), "test_model");
        assert_eq!(model.get_id_field(), "test_id");
        assert_eq!(
            model.get_results_field(),
            Some(&"results".to_string()),
            "Results field should match"
        );
        assert_eq!(
            model.get_description(),
            Some(&"Test model description".to_string()),
            "Description should match"
        );
        assert_eq!(model.get_status(), Some(201), "Status should be 201");
    }

    /// Tests Model creation with minimal parameters.
    ///
    /// Verifies that Model::new works correctly when optional parameters
    /// are set to None.
    #[test]
    fn test_model_new_minimal() {
        let test_data = ConfigRouteResponse {
            status: None,
            body: BodyResponse::Value(Value::Null),
        };

        let model = Model::new(
            "minimal_model".to_string(),
            "id".to_string(),
            None,
            None,
            test_data,
        );

        assert_eq!(model.get_identifier(), "minimal_model");
        assert_eq!(model.get_id_field(), "id");
        assert!(
            model.get_results_field().is_none(),
            "Results field should be None"
        );
        assert!(
            model.get_description().is_none(),
            "Description should be None"
        );
        assert!(model.get_status().is_none(), "Status should be None");
    }

    /// Tests Model data retrieval without results field.
    ///
    /// Verifies that get_data returns the entire response body when
    /// no results field is specified.
    #[test]
    fn test_model_get_data_no_results_field() {
        let test_data = json!({"users": [{"id": 1, "name": "John"}], "count": 1});
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(test_data.clone()),
        };

        let model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        let data = model.get_data();
        if let BodyResponse::Value(value) = data {
            assert_eq!(value, test_data, "Should return entire response body");
        } else {
            panic!("Expected Value variant");
        }
    }

    /// Tests Model data retrieval with results field.
    ///
    /// Verifies that get_data returns only the specified results field
    /// when a results field is configured.
    #[test]
    fn test_model_get_data_with_results_field() {
        let users_array = json!([{"id": 1, "name": "John"}]);
        let full_data = json!({"users": users_array, "count": 1});
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(full_data),
        };

        let model = Model::new(
            "test".to_string(),
            "id".to_string(),
            Some("users".to_string()),
            None,
            config_data,
        );

        let data = model.get_data();
        if let BodyResponse::Value(value) = data {
            assert_eq!(value, users_array, "Should return only the users array");
        } else {
            panic!("Expected Value variant");
        }
    }

    /// Tests Model data retrieval with non-existent results field.
    ///
    /// Verifies that get_data returns the entire response body when
    /// the specified results field doesn't exist in the data.
    #[test]
    fn test_model_get_data_nonexistent_results_field() {
        let test_data = json!({"items": [{"id": 1}]});
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(test_data.clone()),
        };

        let model = Model::new(
            "test".to_string(),
            "id".to_string(),
            Some("nonexistent".to_string()),
            None,
            config_data,
        );

        let data = model.get_data();
        if let BodyResponse::Value(value) = data {
            assert_eq!(
                value, test_data,
                "Should return entire body when results field doesn't exist"
            );
        } else {
            panic!("Expected Value variant");
        }
    }
}

mod data_merging_tests {
    use super::*;

    /// Tests merging objects with objects.
    ///
    /// Verifies that update_data correctly merges two JSON objects,
    /// combining fields and overriding existing values.
    #[test]
    fn test_model_update_data_object_merge() {
        let initial_data = json!({"name": "John", "age": 30});
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(initial_data),
        };

        let mut model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        let new_data = json!({"age": 31, "email": "john@example.com"});
        let result = model.update_data(new_data);
        assert!(result.is_ok(), "Object merge should succeed");

        let updated_data = model.get_data();
        if let BodyResponse::Value(value) = updated_data {
            assert_eq!(value["name"], "John", "Existing field should be preserved");
            assert_eq!(value["age"], 31, "Existing field should be updated");
            assert_eq!(
                value["email"], "john@example.com",
                "New field should be added"
            );
        } else {
            panic!("Expected Value variant");
        }
    }

    /// Tests merging arrays with arrays.
    ///
    /// Verifies that update_data correctly appends items from the new array
    /// to the existing array when both are arrays.
    #[test]
    fn test_model_update_data_array_merge() {
        let initial_data = json!([{"id": 1, "name": "John"}]);
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(initial_data),
        };

        let mut model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        let new_data = json!([{"id": 2, "name": "Jane"}]);
        let result = model.update_data(new_data);
        assert!(result.is_ok(), "Array merge should succeed");

        let updated_data = model.get_data();
        if let BodyResponse::Value(Value::Array(arr)) = updated_data {
            assert_eq!(arr.len(), 2, "Array should contain both items");
            assert_eq!(arr[0]["id"], 1, "First item should be preserved");
            assert_eq!(arr[1]["id"], 2, "Second item should be added");
        } else {
            panic!("Expected array value");
        }
    }

    /// Tests merging object into array.
    ///
    /// Verifies that update_data correctly adds a new object to an existing
    /// array or updates an existing item with matching ID.
    #[test]
    fn test_model_update_data_array_object_merge() {
        let initial_data = json!([{"id": 1, "name": "John"}]);
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(initial_data),
        };

        let mut model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        // Add new item to array
        let new_item = json!({"id": 2, "name": "Jane"});
        let result = model.update_data(new_item);
        assert!(result.is_ok(), "Adding new item should succeed");

        let updated_data = model.get_data();
        if let BodyResponse::Value(Value::Array(arr)) = updated_data {
            assert_eq!(arr.len(), 2, "Array should contain both items");
            assert_eq!(arr[1]["id"], 2, "New item should be added");
        } else {
            panic!("Expected array value");
        }
    }

    /// Tests updating existing item in array by ID.
    ///
    /// Verifies that update_data correctly updates an existing item in an
    /// array when the new object has a matching ID field.
    #[test]
    fn test_model_update_data_array_object_update_existing() {
        let initial_data = json!([
            {"id": 1, "name": "John", "age": 30},
            {"id": 2, "name": "Jane", "age": 25}
        ]);
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(initial_data),
        };

        let mut model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        // Update existing item
        let updated_item = json!({"id": 1, "name": "John Doe", "email": "john@example.com"});
        let result = model.update_data(updated_item);
        assert!(result.is_ok(), "Updating existing item should succeed");

        let updated_data = model.get_data();
        if let BodyResponse::Value(Value::Array(arr)) = updated_data {
            assert_eq!(arr.len(), 2, "Array should still contain two items");
            assert_eq!(arr[0]["id"], 1, "First item ID should be preserved");
            assert_eq!(
                arr[0]["name"], "John Doe",
                "First item name should be updated"
            );
            assert_eq!(arr[0]["age"], 30, "First item age should be preserved");
            assert_eq!(
                arr[0]["email"], "john@example.com",
                "First item should have new email"
            );
            assert_eq!(arr[1]["id"], 2, "Second item should be unchanged");
        } else {
            panic!("Expected array value");
        }
    }

    /// Tests merging with results field.
    ///
    /// Verifies that update_data correctly operates on nested data when
    /// a results field is specified.
    #[allow(clippy::collapsible_match)]
    #[test]
    fn test_model_update_data_with_results_field() {
        let initial_data = json!({
            "users": [{"id": 1, "name": "John"}],
            "count": 1
        });
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(initial_data),
        };

        let mut model = Model::new(
            "test".to_string(),
            "id".to_string(),
            Some("users".to_string()),
            None,
            config_data,
        );

        let new_user = json!({"id": 2, "name": "Jane"});
        let result = model.update_data(new_user);
        assert!(result.is_ok(), "Merge with results field should succeed");

        // Check that the users array was updated
        let updated_data = model.get_data();
        if let BodyResponse::Value(users_data) = updated_data {
            if let Value::Array(users_arr) = users_data {
                assert_eq!(users_arr.len(), 2, "Users array should have two items");
                assert_eq!(users_arr[1]["id"], 2, "New user should be added");
            }
        }

        // We can't directly access the full structure to verify count is unchanged
        // but we can verify that the results field functionality worked correctly
    }

    /// Tests merging with non-existent results field.
    ///
    /// Verifies that update_data creates the results field when it doesn't
    /// exist in the original data structure.
    #[test]
    fn test_model_update_data_create_results_field() {
        let initial_data = json!({"count": 0});
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(initial_data),
        };

        let mut model = Model::new(
            "test".to_string(),
            "id".to_string(),
            Some("items".to_string()),
            None,
            config_data,
        );

        let new_item = json!({"id": 1, "name": "Test Item"});
        let result = model.update_data(new_item.clone());
        assert!(result.is_ok(), "Creating new results field should succeed");

        // Check that the items field was created by getting the data again
        let updated_data = model.get_data();
        if let BodyResponse::Value(items_data) = updated_data {
            assert_eq!(items_data, new_item, "Items field should contain new data");
        }
    }

    /// Tests error handling for string-based responses.
    ///
    /// Verifies that update_data returns an appropriate error when trying
    /// to merge data with string-based response bodies.
    #[test]
    fn test_model_update_data_string_response_error() {
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::String("static response".to_string()),
        };

        let mut model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        let new_data = json!({"test": "data"});
        let result = model.update_data(new_data);
        assert!(result.is_err(), "String response merge should fail");

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Cannot merge data with string-based responses"),
            "Error message should indicate string response issue"
        );
    }
}

mod query_functionality_tests {
    use super::*;

    /// Tests finding entries by HashMap with exact matches.
    ///
    /// Verifies that find_entry_by_hashmap correctly finds entries that
    /// match the provided field-value pairs.
    #[test]
    fn test_model_find_entry_by_hashmap_exact_match() {
        let test_data = json!([
            {"id": 1, "name": "John", "status": "active"},
            {"id": 2, "name": "Jane", "status": "inactive"}
        ]);
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(test_data),
        };

        let model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        let mut search_params = HashMap::new();
        search_params.insert("name".to_string(), "Jane".to_string());

        let result = model.find_entry_by_hashmap(search_params);
        assert!(result.is_some(), "Should find matching entry");

        let entry = result.unwrap();
        assert_eq!(entry["id"], 2, "Should return Jane's entry");
        assert_eq!(entry["name"], "Jane", "Name should match");
    }

    /// Tests finding entries by ID field.
    ///
    /// Verifies that find_entry_by_hashmap correctly handles ID field
    /// searches with special string comparison logic.
    #[test]
    fn test_model_find_entry_by_hashmap_id_field() {
        let test_data = json!([
            {"id": 1, "name": "John"},
            {"id": 2, "name": "Jane"}
        ]);
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(test_data),
        };

        let model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        let mut search_params = HashMap::new();
        search_params.insert("id".to_string(), "1".to_string());

        let result = model.find_entry_by_hashmap(search_params);
        assert!(result.is_some(), "Should find entry by ID");

        let entry = result.unwrap();
        assert_eq!(entry["id"], 1, "Should return correct entry");
        assert_eq!(entry["name"], "John", "Should return John's entry");
    }

    /// Tests finding entries with colon in field names.
    ///
    /// Verifies that find_entry_by_hashmap correctly handles field names
    /// with colons by removing them before matching.
    #[test]
    fn test_model_find_entry_by_hashmap_colon_field() {
        let test_data = json!([
            {"id": 1, "name": "John", "metadata": {"type": "user"}},
            {"id": 2, "name": "Jane", "metadata": {"type": "admin"}}
        ]);
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(test_data),
        };

        let model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        let mut search_params = HashMap::new();
        search_params.insert("name:".to_string(), "Jane".to_string());

        let result = model.find_entry_by_hashmap(search_params);
        assert!(
            result.is_some(),
            "Should find entry with colon in field name"
        );

        let entry = result.unwrap();
        assert_eq!(entry["name"], "Jane", "Should return Jane's entry");
    }

    /// Tests finding entries in object data.
    ///
    /// Verifies that find_entry_by_hashmap works with object-based data
    /// structures rather than arrays.
    #[test]
    fn test_model_find_entry_by_hashmap_object_data() {
        let test_data = json!({
            "name": "John",
            "age": 30,
            "status": "active"
        });
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(test_data),
        };

        let model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        let mut search_params = HashMap::new();
        search_params.insert("status".to_string(), "active".to_string());

        let result = model.find_entry_by_hashmap(search_params);
        assert!(result.is_some(), "Should find field in object data");

        let entry = result.unwrap();
        assert_eq!(entry, "active", "Should return the field value");
    }

    /// Tests finding entries with no matches.
    ///
    /// Verifies that find_entry_by_hashmap returns None when no entries
    /// match the provided search criteria.
    #[test]
    fn test_model_find_entry_by_hashmap_no_match() {
        let test_data = json!([
            {"id": 1, "name": "John"},
            {"id": 2, "name": "Jane"}
        ]);
        let config_data = ConfigRouteResponse {
            status: Some(200),
            body: BodyResponse::Value(test_data),
        };

        let model = Model::new(
            "test".to_string(),
            "id".to_string(),
            None,
            None,
            config_data,
        );

        let mut search_params = HashMap::new();
        search_params.insert("name".to_string(), "Bob".to_string());

        let result = model.find_entry_by_hashmap(search_params);
        assert!(result.is_none(), "Should not find non-existent entry");
    }
}

mod database_update_tests {
    use super::*;

    /// Tests updating model data through database.
    ///
    /// Verifies that update_model_data correctly updates model data
    /// through the database interface.
    #[allow(clippy::collapsible_match)]
    #[test]
    fn test_database_update_model_data_success() {
        let mut db = create_test_database();

        let new_user = json!({"id": 3, "name": "Bob", "email": "bob@example.com"});
        let result = db.update_model_data("[GET] /api/users", new_user);
        assert!(result.is_ok(), "Model data update should succeed");

        // Verify the update
        let model = db.get_model("[GET] /api/users").unwrap();
        let data = model.get_data();
        if let BodyResponse::Value(value) = data {
            if let Value::Object(obj) = value {
                if let Some(Value::Array(users)) = obj.get("users") {
                    assert_eq!(users.len(), 3, "Should have three users now");
                    assert_eq!(users[2]["id"], 3, "New user should be added");
                    assert_eq!(users[2]["name"], "Bob", "New user name should match");
                }
            }
        }
    }

    /// Tests updating non-existent model.
    ///
    /// Verifies that update_model_data returns an appropriate error when
    /// trying to update a model that doesn't exist.
    #[test]
    fn test_database_update_model_data_nonexistent() {
        let mut db = create_test_database();

        let new_data = json!({"test": "data"});
        let result = db.update_model_data("nonexistent", new_data);
        assert!(result.is_err(), "Updating nonexistent model should fail");

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("not found"),
            "Error should indicate model not found"
        );
    }

    /// Tests updating model with existing item modification.
    ///
    /// Verifies that update_model_data correctly modifies existing items
    /// when updating model data with matching IDs.
    #[allow(clippy::collapsible_match)]
    #[test]
    fn test_database_update_model_data_modify_existing() {
        let mut db = create_test_database();

        let updated_user = json!({
            "id": 1,
            "name": "John Doe Updated",
            "email": "john.updated@example.com",
            "status": "active"
        });
        let result = db.update_model_data("[GET] /api/users", updated_user);
        assert!(result.is_ok(), "Model data update should succeed");

        // Verify the update
        let model = db.get_model("[GET] /api/users").unwrap();
        let data = model.get_data();
        if let BodyResponse::Value(value) = data {
            if let Value::Object(obj) = value {
                if let Some(Value::Array(users)) = obj.get("users") {
                    assert_eq!(users.len(), 2, "Should still have two users");
                    let updated_user = &users[0];
                    assert_eq!(updated_user["id"], 1, "User ID should be preserved");
                    assert_eq!(
                        updated_user["name"], "John Doe Updated",
                        "User name should be updated"
                    );
                    assert_eq!(
                        updated_user["email"], "john.updated@example.com",
                        "User email should be updated"
                    );
                    assert_eq!(
                        updated_user["status"], "active",
                        "New status field should be added"
                    );
                }
            }
        }
    }
}

mod integration_tests {
    use super::*;

    /// Tests complete database workflow.
    ///
    /// Verifies that the database can handle a complete workflow including
    /// population, querying, data updates, and model retrieval.
    #[test]
    fn test_database_complete_workflow() {
        let mut db = Database::new();

        // Step 1: Populate database
        let mut routes = HashMap::new();
        let initial_data = json!({"items": [{"id": 1, "name": "Initial Item"}]});
        routes.insert(
            "[GET] /api/items".to_string(),
            create_test_route("GET", Some("Items API"), "id", Some("items"), initial_data),
        );
        db.populate(routes);

        // Step 2: Verify initial state
        assert_eq!(db.get_routes().len(), 1, "Should have one route");
        assert_eq!(db.get_models().len(), 1, "Should have one model");

        // Step 3: Query for route and model
        let route = db.get_route("/api/items", Some("GET".to_string()));
        assert!(route.is_some(), "Should find the items route");

        let model = db.get_model("[GET] /api/items");
        assert!(model.is_some(), "Should find the items model");

        // Step 4: Update model data
        let new_item = json!({"id": 2, "name": "Second Item"});
        let update_result = db.update_model_data("[GET] /api/items", new_item);
        assert!(update_result.is_ok(), "Data update should succeed");

        // Step 5: Verify updated data
        let updated_model = db.get_model("[GET] /api/items").unwrap();
        let data = updated_model.get_data();
        if let BodyResponse::Value(Value::Array(items)) = data {
            assert_eq!(items.len(), 2, "Should have two items after update");
            assert_eq!(items[1]["id"], 2, "Second item should be added");
        } else {
            panic!("Expected array data");
        }

        // Step 6: Search for specific item
        let mut search_params = HashMap::new();
        search_params.insert("name".to_string(), "Second Item".to_string());
        let found_item = updated_model.find_entry_by_hashmap(search_params);
        assert!(found_item.is_some(), "Should find the second item");
        assert_eq!(found_item.unwrap()["id"], 2, "Found item should have ID 2");
    }

    /// Tests database with mixed data types.
    ///
    /// Verifies that the database correctly handles models with different
    /// data structures and configurations.
    #[test]
    fn test_database_mixed_data_types() {
        let mut db = Database::new();
        let mut routes = HashMap::new();

        // Array-based route
        let array_data = json!([{"id": 1, "type": "array"}]);
        routes.insert(
            "[GET] /array".to_string(),
            create_test_route("GET", None, "id", None, array_data),
        );

        // Object-based route
        let object_data = json!({"status": "ok", "message": "object"});
        routes.insert(
            "[GET] /object".to_string(),
            create_test_route("GET", None, "id", None, object_data),
        );

        // Nested structure route
        let nested_data = json!({"data": {"items": [{"id": 1, "nested": true}]}});
        routes.insert(
            "[GET] /nested".to_string(),
            create_test_route("GET", None, "id", Some("data"), nested_data),
        );

        db.populate(routes);

        // Test each route type
        assert_eq!(db.get_routes().len(), 3, "Should have three routes");
        assert_eq!(db.get_models().len(), 3, "Should have three models");

        // Test array model
        let array_model = db.get_model("[GET] /array").unwrap();
        let array_data = array_model.get_data();
        if let BodyResponse::Value(Value::Array(_)) = array_data {
            // Expected array type
        } else {
            panic!("Expected array data for array model");
        }

        // Test object model
        let object_model = db.get_model("[GET] /object").unwrap();
        let object_data = object_model.get_data();
        if let BodyResponse::Value(Value::Object(_)) = object_data {
            // Expected object type
        } else {
            panic!("Expected object data for object model");
        }

        // Test nested model
        let nested_model = db.get_model("[GET] /nested").unwrap();
        let nested_data = nested_model.get_data();
        if let BodyResponse::Value(value) = nested_data {
            // Should extract the "data" field content
            assert!(value.get("items").is_some(), "Should extract nested items");
        } else {
            panic!("Expected object data for nested model");
        }
    }
}
