//! Database module for managing routes and models in the JSON Echo application.
//!
//! This module provides the core data structures and functionality for handling
//! configuration routes and their associated models. It serves as an in-memory
//! database that stores route configurations and provides methods to query and
//! retrieve data from these configurations.
//!
//! ## What
//!
//! The module defines two main structures:
//! - `Database`: A container that holds route configurations and their corresponding models
//! - `Model`: A representation of a route's data structure with metadata for data access
//!
//! ## How
//!
//! The database works by:
//! 1. Storing route configurations in a HashMap for fast lookup by identifier
//! 2. Converting route configurations into models with extracted metadata
//! 3. Providing query methods to find specific data entries within models
//! 4. Supporting both object and array-based data structures
//!
//! ## Why
//!
//! This design enables:
//! - Fast route lookup and data retrieval
//! - Flexible data querying with support for custom ID fields
//! - Separation of configuration from runtime data access
//! - Type-safe access to JSON data structures
//!
//! # Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use json_echo_core::{Database, ConfigRoute};
//!
//! let mut db = Database::new();
//! let mut routes = HashMap::new();
//!
//! // routes would be populated with ConfigRoute instances
//! db.populate(routes);
//!
//! // Query for a specific route
//! if let Some(route) = db.get_route("users") {
//!     println!("Found route: {:?}", route);
//! }
//! ```

use std::collections::HashMap;

use serde_json::{Map, Value, json};

use crate::{ConfigRoute, ConfigRouteResponse, config::BodyResponse};

/// An in-memory database that manages route configurations and their associated models.
///
/// The `Database` struct serves as the central repository for route configurations
/// and provides methods to query and retrieve data. It maintains both the original
/// route configurations and processed models for efficient data access.
///
/// # Fields
///
/// * `routes` - A HashMap containing route configurations indexed by their identifier
/// * `models` - A vector of processed models derived from the route configurations
///
/// # Examples
///
/// ```rust
/// use json_echo_core::Database;
/// use std::collections::HashMap;
///
/// let mut db = Database::new();
/// let routes = HashMap::new(); // Would contain actual ConfigRoute instances
/// db.populate(routes);
///
/// // Access routes and models
/// let route_keys = db.get_routes();
/// let models = db.get_models();
/// ```
#[derive(Debug, Clone)]
pub struct Database {
    /// HashMap storing route configurations indexed by their string identifier
    pub(crate) routes: HashMap<String, ConfigRoute>,
    /// Vector containing all processed models derived from route configurations
    pub(crate) models: Vec<Model>,
}

/// A model representing a processed route configuration with metadata for data access.
///
/// The `Model` struct contains the essential information needed to interact with
/// the data associated with a specific route. It includes metadata such as the
/// identifier field, description, and the actual response data.
///
/// # Fields
///
/// * `identifier` - The unique identifier for this model
/// * `id_field` - The field name used as the primary identifier in the data
/// * `results_field` - Optional field name that contains the actual results data
/// * `description` - Optional human-readable description of the model
/// * `data` - The actual response data configuration
///
/// # Examples
///
/// ```rust
/// use json_echo_core::{Model, ConfigRouteResponse};
/// use serde_json::Value;
///
/// let model = Model::new(
///     "users".to_string(),
///     "id".to_string(),
///     Some("data".to_string()),
///     Some("User management model".to_string()),
///     ConfigRouteResponse {
///         status: Some(200),
///         body: Value::Null,
///     }
/// );
///
/// assert_eq!(model.get_identifier(), "users");
/// assert_eq!(model.get_id_field(), "id");
/// ```
#[derive(Debug, Clone)]
pub struct Model {
    /// The unique string identifier for this model
    pub(crate) identifier: String,
    /// The field name used as the primary identifier in the data structure
    pub(crate) id_field: String,
    /// Optional field name that contains the actual results when data is nested
    pub(crate) results_field: Option<String>,
    /// Optional human-readable description explaining the purpose of this model
    pub(crate) description: Option<String>,
    /// The configuration response data associated with this model
    pub(crate) data: ConfigRouteResponse,
}

impl Database {
    /// Creates a new empty database instance.
    ///
    /// Initializes a new `Database` with empty routes HashMap and models vector.
    /// This is the standard way to create a database before populating it with
    /// route configurations.
    ///
    /// # Returns
    ///
    /// A new `Database` instance with no routes or models
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Database;
    ///
    /// let db = Database::new();
    /// assert_eq!(db.get_routes().len(), 0);
    /// assert_eq!(db.get_models().len(), 0);
    /// ```
    pub fn new() -> Self {
        Database {
            routes: HashMap::new(),
            models: Vec::new(),
        }
    }

    /// Populates the database with route configurations and generates corresponding models.
    ///
    /// This method takes a HashMap of route configurations, stores them internally,
    /// and processes each route to create a corresponding `Model` instance. The models
    /// are generated with default values for missing fields and proper data extraction.
    ///
    /// # Parameters
    ///
    /// * `routes` - A HashMap where keys are route identifiers and values are `ConfigRoute` instances
    ///
    /// # Behavior
    ///
    /// - Replaces any existing routes and models
    /// - Generates models with default ID field "id" if not specified
    /// - Extracts response data or provides empty object as fallback
    /// - Preserves route descriptions and other metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::{Database, ConfigRoute};
    /// use std::collections::HashMap;
    ///
    /// let mut db = Database::new();
    /// let mut routes = HashMap::new();
    ///
    /// // routes.insert("users".to_string(), some_config_route);
    /// db.populate(routes);
    ///
    /// // Now database contains routes and generated models
    /// assert!(!db.get_routes().is_empty());
    /// ```
    #[allow(clippy::map_unwrap_or)]
    pub fn populate(&mut self, routes: HashMap<String, ConfigRoute>) {
        self.routes = routes;

        for (key, route) in &self.routes {
            let model = Model {
                identifier: key.clone(),
                id_field: route.id_field.clone().unwrap_or_else(|| String::from("id")),
                description: route.description.clone(),
                results_field: route.results_field.clone(),
                data: match &route.response {
                    crate::ConfigResponse::ConfigRouteResponse(response) => response.clone(),
                    _ => ConfigRouteResponse {
                        status: Some(200),
                        body: BodyResponse::Value(Value::Object(Map::new())),
                    },
                },
            };

            self.models.push(model);
        }
    }

    /// Retrieves a route configuration by its identifier.
    ///
    /// Performs a lookup in the internal routes HashMap using the provided identifier.
    /// Returns a reference to the `ConfigRoute` if found, or `None` if the identifier
    /// does not exist in the database.
    ///
    /// # Parameters
    ///
    /// * `identifier` - The string identifier of the route to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(&ConfigRoute)` - If a route with the given identifier exists
    /// * `None` - If no route is found with the given identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Database;
    ///
    /// let db = Database::new();
    /// // Assuming database has been populated
    ///
    /// if let Some(route) = db.get_route("users") {
    ///     println!("Found route for users");
    /// } else {
    ///     println!("No route found for users");
    /// }
    /// ```
    pub fn get_route(&self, identifier: &str) -> Option<&ConfigRoute> {
        self.routes.get(identifier)
    }

    /// Returns a vector of references to all route identifiers in the database.
    ///
    /// Collects all keys from the internal routes HashMap and returns them as
    /// a vector of string references. This provides a way to enumerate all
    /// available routes without accessing the full route configurations.
    ///
    /// # Returns
    ///
    /// A `Vec<&String>` containing references to all route identifiers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Database;
    ///
    /// let db = Database::new();
    /// // Assuming database has been populated with routes
    ///
    /// let routes = db.get_routes();
    /// for route_id in routes {
    ///     println!("Available route: {}", route_id);
    /// }
    /// ```
    pub fn get_routes(&self) -> Vec<&String> {
        let mut routes = vec![];
        for key in self.routes.keys() {
            routes.push(key);
        }
        routes
    }

    /// Returns a reference to the vector containing all models in the database.
    ///
    /// Provides direct access to the internal models vector, allowing callers
    /// to iterate over or inspect all models without individual lookups.
    ///
    /// # Returns
    ///
    /// A reference to `Vec<Model>` containing all models in the database
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Database;
    ///
    /// let db = Database::new();
    /// // Assuming database has been populated
    ///
    /// let models = db.get_models();
    /// println!("Database contains {} models", models.len());
    ///
    /// for model in models {
    ///     println!("Model: {}", model.get_identifier());
    /// }
    /// ```
    pub fn get_models(&self) -> &Vec<Model> {
        &self.models
    }

    /// Retrieves a specific model by its identifier.
    ///
    /// Searches through the models vector to find a model with the specified
    /// identifier. This method performs a linear search and returns the first
    /// matching model.
    ///
    /// # Parameters
    ///
    /// * `identifier` - The string identifier of the model to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(&Model)` - If a model with the given identifier exists
    /// * `None` - If no model is found with the given identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Database;
    ///
    /// let db = Database::new();
    /// // Assuming database has been populated
    ///
    /// if let Some(model) = db.get_model("users") {
    ///     println!("Found model: {}", model.get_identifier());
    ///     println!("ID field: {}", model.get_id_field());
    /// }
    /// ```
    pub fn get_model(&self, identifier: &str) -> Option<&Model> {
        self.models
            .iter()
            .find(|model| model.identifier == identifier)
    }
}

impl Model {
    /// Creates a new model instance with the specified parameters.
    ///
    /// Constructs a new `Model` with all required fields. This is typically used
    /// internally by the database population process, but can also be used to
    /// create models manually for testing or specialized use cases.
    ///
    /// # Parameters
    ///
    /// * `identifier` - The unique identifier for this model
    /// * `id_field` - The field name used as the primary identifier in the data
    /// * `results_field` - Optional field name containing the actual results data
    /// * `description` - Optional human-readable description of the model
    /// * `data` - The configuration response data for this model
    ///
    /// # Returns
    ///
    /// A new `Model` instance with the provided configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::{Model, ConfigRouteResponse};
    /// use serde_json::Value;
    ///
    /// let model = Model::new(
    ///     "products".to_string(),
    ///     "product_id".to_string(),
    ///     Some("items".to_string()),
    ///     Some("Product catalog model".to_string()),
    ///     ConfigRouteResponse {
    ///         status: Some(200),
    ///         body: Value::Null,
    ///     }
    /// );
    /// ```
    pub fn new(
        identifier: String,
        id_field: String,
        results_field: Option<String>,
        description: Option<String>,
        data: ConfigRouteResponse,
    ) -> Self {
        Model {
            identifier,
            id_field,
            results_field,
            description,
            data,
        }
    }

    /// Returns the identifier of this model.
    ///
    /// Provides access to the model's unique identifier string that is used
    /// to distinguish this model from others in the database.
    ///
    /// # Returns
    ///
    /// A string slice containing the model's identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Model;
    ///
    /// // Assuming model has been created
    /// let model_id = model.get_identifier();
    /// println!("Working with model: {}", model_id);
    /// ```
    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    /// Returns the description of this model, if available.
    ///
    /// Provides access to the optional human-readable description that explains
    /// the purpose or content of this model.
    ///
    /// # Returns
    ///
    /// * `Some(&String)` - If a description is available
    /// * `None` - If no description was provided for this model
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Model;
    ///
    /// // Assuming model has been created
    /// if let Some(desc) = model.get_description() {
    ///     println!("Model description: {}", desc);
    /// } else {
    ///     println!("No description available");
    /// }
    /// ```
    pub fn get_description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// Returns the data associated with this model.
    ///
    /// Extracts and returns the actual data content from the model. If a results
    /// field is specified, it attempts to extract that specific field from the
    /// response body. Otherwise, it returns the entire response body.
    ///
    /// # Returns
    ///
    /// A reference to the `Value` containing the model's data
    ///
    /// # Behavior
    ///
    /// - If `results_field` is specified and exists in the response body, returns that field's value
    /// - Otherwise, returns the entire response body
    /// - Handles both object and non-object response structures
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Model;
    /// use serde_json::Value;
    ///
    /// // Assuming model has been created
    /// let data = model.get_data();
    ///
    /// match data {
    ///     Value::Array(arr) => println!("Data contains {} items", arr.len()),
    ///     Value::Object(obj) => println!("Data contains {} fields", obj.len()),
    ///     _ => println!("Data is a simple value"),
    /// }
    /// ```
    #[allow(clippy::collapsible_match)]
    pub fn get_data(&self) -> BodyResponse {
        if let Some(results_field) = &self.results_field {
            // Only if body is type Value
            if let BodyResponse::Value(body) = &self.data.body {
                if let Value::Object(map) = body {
                    if let Some(value) = map.get(results_field) {
                        return BodyResponse::Value(value.clone());
                    }
                }
            }
        }

        self.data.body.clone()
    }

    /// Returns the HTTP status code associated with this model's response.
    ///
    /// Provides access to the HTTP status code that should be returned when
    /// this model's data is accessed through the API.
    ///
    /// # Returns
    ///
    /// * `Some(u16)` - The HTTP status code if specified
    /// * `None` - If no status code was configured
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Model;
    ///
    /// // Assuming model has been created
    /// match model.get_status() {
    ///     Some(200) => println!("Success response"),
    ///     Some(404) => println!("Not found response"),
    ///     Some(status) => println!("Response status: {}", status),
    ///     None => println!("No status code specified"),
    /// }
    /// ```
    pub fn get_status(&self) -> Option<u16> {
        self.data.status
    }

    /// Returns the name of the field used as the primary identifier in the data.
    ///
    /// Provides access to the field name that serves as the unique identifier
    /// within the model's data structure. This is used for querying and filtering
    /// operations.
    ///
    /// # Returns
    ///
    /// A string slice containing the ID field name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Model;
    ///
    /// // Assuming model has been created
    /// let id_field = model.get_id_field();
    /// println!("Using '{}' as the identifier field", id_field);
    /// ```
    pub fn get_id_field(&self) -> &str {
        &self.id_field
    }

    /// Returns the name of the results field, if specified.
    ///
    /// Provides access to the optional field name that contains the actual
    /// results data when the response structure has nested data.
    ///
    /// # Returns
    ///
    /// * `Some(&String)` - If a results field is configured
    /// * `None` - If no results field was specified
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Model;
    ///
    /// // Assuming model has been created
    /// if let Some(results_field) = model.get_results_field() {
    ///     println!("Results are nested under: {}", results_field);
    /// } else {
    ///     println!("Results are at the root level");
    /// }
    /// ```
    pub fn get_results_field(&self) -> Option<&String> {
        self.results_field.as_ref()
    }

    /// Searches for a data entry that matches the provided field-value pairs.
    ///
    /// Performs a search through the model's data to find an entry that matches
    /// any of the provided key-value pairs. The method handles both object and
    /// array data structures and supports special handling for ID field matches.
    ///
    /// # Parameters
    ///
    /// * `map` - A HashMap containing field names as keys and their expected values as strings
    ///
    /// # Returns
    ///
    /// * `Some(Value)` - The matching data entry if found
    /// * `None` - If no matching entry is found
    ///
    /// # Behavior
    ///
    /// - Removes colons from field names before matching
    /// - Provides special handling for ID field matches with string comparison
    /// - Supports both exact JSON value matches and string-based comparisons
    /// - Works with both object-based and array-based data structures
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Model;
    /// use std::collections::HashMap;
    ///
    /// // Assuming model has been created with user data
    /// let mut search_params = HashMap::new();
    /// search_params.insert("id".to_string(), "123".to_string());
    /// search_params.insert("name".to_string(), "John".to_string());
    ///
    /// if let Some(entry) = model.find_entry_by_hashmap(search_params) {
    ///     println!("Found matching entry: {}", entry);
    /// } else {
    ///     println!("No matching entry found");
    /// }
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn find_entry_by_hashmap(&self, map: HashMap<String, String>) -> Option<Value> {
        let id_field = self.get_id_field();

        if let BodyResponse::Value(body) = &self.get_data() {
            if let Value::Object(obj) = body {
                for (key, value) in &map {
                    if let Some(val) = obj.get(&key.replace(':', "")) {
                        if key.contains(id_field) && val.to_string().as_str() == value {
                            return Some(val.clone());
                        }

                        if *val == json!(value) {
                            return Some(val.clone());
                        }
                    }
                }
            } else if let Value::Array(arr) = body {
                for item in arr {
                    if let Value::Object(obj) = item {
                        for (key, value) in &map {
                            if let Some(val) = obj.get(&key.replace(':', "")) {
                                if key.contains(id_field) && val.to_string().as_str() == value {
                                    return Some(item.clone());
                                }

                                if *val == json!(value.clone()) {
                                    return Some(item.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}
