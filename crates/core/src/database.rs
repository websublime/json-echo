use std::collections::HashMap;

use serde_json::{Map, Value, json};

use crate::{ConfigRoute, ConfigRouteResponse};

pub struct Database {
    pub(crate) routes: HashMap<String, ConfigRoute>,
    pub(crate) models: Vec<Model>,
}

pub struct Model {
    pub(crate) identifier: String,
    pub(crate) description: Option<String>,
    pub(crate) data: ConfigRouteResponse,
}

impl Database {
    pub fn new() -> Self {
        Database {
            routes: HashMap::new(),
            models: Vec::new(),
        }
    }

    #[allow(clippy::map_unwrap_or)]
    pub fn populate(&mut self, routes: HashMap<String, ConfigRoute>) {
        self.routes = routes;

        for (key, route) in &self.routes {
            let model = Model {
                identifier: key.clone(),
                description: route.description.clone(),
                data: match &route.response {
                    crate::ConfigResponse::ConfigRouteResponse(response) => response.clone(),
                    _ => ConfigRouteResponse {
                        status: Some(200),
                        body: Value::Object(Map::new()),
                    },
                },
            };

            self.models.push(model);
        }
    }

    pub fn get_route(&self, identifier: &str) -> Option<&ConfigRoute> {
        self.routes.get(identifier)
    }

    pub fn get_routes(&self) -> Vec<&String> {
        let mut routes = vec![];
        for key in self.routes.keys() {
            routes.push(key);
        }
        routes
    }

    pub fn get_model(&self, identifier: &str) -> Option<&Model> {
        self.models
            .iter()
            .find(|model| model.identifier == identifier)
    }
}

impl Model {
    pub fn new(identifier: String, description: Option<String>, data: ConfigRouteResponse) -> Self {
        Model {
            identifier,
            description,
            data,
        }
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    pub fn get_description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn get_data(&self) -> &Value {
        &self.data.body
    }

    pub fn get_status(&self) -> Option<u16> {
        self.data.status
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn find_entry_by_hashmap(&self, map: HashMap<String, String>) -> Option<Value> {
        if let Value::Object(obj) = &self.data.body {
            for (key, value) in &map {
                if let Some(val) = obj.get(&key.replace(':', "")) {
                    if key.contains("id") && val.to_string().as_str() == value {
                        return Some(val.clone());
                    }

                    if *val == json!(value) {
                        return Some(val.clone());
                    }
                }
            }
        } else if let Value::Array(arr) = &self.data.body {
            for item in arr {
                if let Value::Object(obj) = item {
                    for (key, value) in &map {
                        if let Some(val) = obj.get(&key.replace(':', "")) {
                            if key.contains("id") && val.to_string().as_str() == value {
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
        None
    }
}
