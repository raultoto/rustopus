use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use regex::Regex;
use anyhow::{Result, Context};
use tracing::{debug, instrument};
use crate::config::types::EndpointConfig;
use super::{HttpHandler, HttpClient};

#[derive(Debug, Clone)]
pub struct Route {
    pub(crate) pattern: Regex,
    pub(crate) handler: Arc<dyn HttpHandler>,
    pub(crate) config: EndpointConfig,
}

#[derive(Default)]
pub struct HttpRouter {
    routes: HashMap<String, Route>,
}

impl HttpRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    #[instrument(skip(self, handler))]
    pub fn add_route<H>(&mut self, path: &str, config: EndpointConfig, handler: H) -> Result<()>
    where
        H: HttpHandler + 'static,
    {
        let pattern = path_to_regex(path)?;
        let route = Route {
            pattern,
            handler: Arc::new(handler),
            config,
        };

        debug!(path = %path, "Adding route");
        self.routes.insert(path.to_string(), route);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn match_route(&self, path: &str) -> Option<(&Route, HashMap<String, String>)> {
        for route in self.routes.values() {
            if let Some(captures) = route.pattern.captures(path) {
                let mut params = HashMap::new();
                for name in route.pattern.capture_names().flatten() {
                    if let Some(value) = captures.name(name) {
                        params.insert(name.to_string(), value.as_str().to_string());
                    }
                }
                return Some((route, params));
            }
        }
        None
    }

    pub fn routes(&self) -> &HashMap<String, Route> {
        &self.routes
    }
}

fn path_to_regex(path: &str) -> Result<Regex> {
    let mut pattern = String::with_capacity(path.len() * 2);
    pattern.push('^');

    for segment in path.split('/') {
        pattern.push('/');
        if segment.starts_with(':') {
            let param_name = &segment[1..];
            pattern.push_str(&format!("(?P<{}>\\w+)", param_name));
        } else if segment == "*" {
            pattern.push_str(".*");
        } else {
            pattern.push_str(&regex::escape(segment));
        }
    }

    pattern.push('$');
    Regex::new(&pattern).context("Failed to compile route pattern")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{BackendConfig, BackendProtocol};

    #[test]
    fn test_path_to_regex() {
        let cases = vec![
            ("/users", "^/users$"),
            ("/users/:id", "^/users/(?P<id>\\w+)$"),
            ("/users/:id/posts", "^/users/(?P<id>\\w+)/posts$"),
            ("/users/*", "^/users/.*$"),
        ];

        for (path, expected) in cases {
            let regex = path_to_regex(path).unwrap();
            assert_eq!(regex.as_str(), expected);
        }
    }

    #[test]
    fn test_route_matching() {
        let mut router = HttpRouter::new();
        let config = EndpointConfig {
            path: "/users/:id".to_string(),
            method: "GET".to_string(),
            backend: vec![BackendConfig {
                url: "http://backend/users".to_string(),
                method: Some("GET".to_string()),
                timeout: None,
                circuit_breaker: None,
                retry: None,
                protocol: BackendProtocol::Rest,
            }],
            timeout: None,
            cache_ttl: None,
            rate_limit: None,
            auth_required: false,
            protocol: crate::config::types::GatewayProtocol::Rest,
            guards: vec![],
        };

        router.add_route("/users/:id", config.clone(), HttpClient::new(config.backend[0].clone()).unwrap()).unwrap();

        let (_, params) = router.match_route("/users/123").unwrap();
        assert_eq!(params.get("id").unwrap(), "123");
    }
} 