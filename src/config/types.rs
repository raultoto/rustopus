use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::collections::HashMap;
use crate::config::{loader, validator};

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => d.as_secs().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = Option::<u64>::deserialize(deserializer)?;
        Ok(secs.map(Duration::from_secs))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
    pub security: SecurityConfig,
    pub plugins: PluginsConfig,
    pub endpoints: Vec<EndpointConfig>,
    pub cluster: ClusterConfig,
    pub tls: TlsConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub ws_port: Option<u16>,
    pub workers: usize,
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    #[serde(default = "default_max_request_size")]
    pub max_request_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    #[serde(default)]
    pub file_output: Option<String>,
    #[serde(default)]
    pub json_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub cors: CorsConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub waf: WafConfig,
    #[serde(default)]
    pub rbac: RbacConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    #[serde(with = "duration_serde")]
    pub max_age: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_second: u32,
    pub burst: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub enabled: bool,
    pub jwt_secret: Option<String>,
    pub jwt_issuer: Option<String>,
    pub jwt_audience: Option<String>,
    pub oauth: Option<OAuthConfig>,
    pub oidc: Option<OidcConfig>,
    pub api_key: Option<ApiKeyConfig>,
    pub mfa: Option<MfaConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WafConfig {
    pub enabled: bool,
    pub rules_file: Option<String>,
    pub block_mode: bool,
    pub allowed_content_types: Vec<String>,
    pub max_request_size: usize,
    pub max_url_length: usize,
    pub max_header_count: usize,
    pub max_header_size: usize,
    pub blocked_countries: Vec<String>,
    pub blocked_ips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RbacConfig {
    pub enabled: bool,
    pub rules_file: Option<String>,
    pub default_role: String,
    pub roles: HashMap<String, RoleConfig>,
    pub policies: Vec<PolicyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    pub name: String,
    pub permissions: Vec<String>,
    pub inherit_from: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub name: String,
    pub effect: PolicyEffect,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    pub conditions: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub enabled: bool,
    pub providers: HashMap<String, OAuthProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    pub client_id: String,
    pub client_secret: String,
    pub authorize_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub enabled: bool,
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub enabled: bool,
    pub header_name: String,
    pub in_query: bool,
    pub query_param: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    pub enabled: bool,
    pub methods: Vec<MfaMethod>,
    pub enforcement: MfaEnforcement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MfaMethod {
    Totp,
    Sms,
    Email,
    WebAuthn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MfaEnforcement {
    Always,
    RiskBased,
    Optional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub directory: Option<String>,
    #[serde(default)]
    pub wasm_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendProtocol {
    Rest,
    Grpc,
    WebSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum GatewayProtocol {
    Rest,
    WebSocket,
}

impl std::str::FromStr for GatewayProtocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rest" => Ok(Self::Rest),
            "websocket" => Ok(Self::WebSocket),
            _ => Err(anyhow::anyhow!("Invalid protocol: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub path: String,
    pub method: String,
    pub backend: Vec<BackendConfig>,
    #[serde(default)]
    #[serde(with = "option_duration_serde")]
    pub timeout: Option<Duration>,
    #[serde(default)]
    #[serde(with = "option_duration_serde")]
    pub cache_ttl: Option<Duration>,
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,
    #[serde(default)]
    pub auth_required: bool,
    #[serde(default = "default_gateway_protocol")]
    pub protocol: GatewayProtocol,
    #[serde(default)]
    pub guards: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub url: String,
    pub method: Option<String>,
    #[serde(with = "option_duration_serde")]
    pub timeout: Option<Duration>,
    #[serde(default)]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
    #[serde(default = "default_backend_protocol")]
    pub protocol: BackendProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub threshold: u32,
    #[serde(with = "duration_serde")]
    pub window: Duration,
    pub min_requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub attempts: u32,
    #[serde(with = "duration_serde")]
    pub backoff: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    #[serde(default)]
    pub enabled: bool,
    pub discovery_method: Option<String>,
    pub discovery_endpoints: Vec<String>,
    pub node_name: Option<String>,
    pub node_role: Option<String>,
    pub sync_interval: Option<Duration>,
    pub leader_election: Option<LeaderElectionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderElectionConfig {
    pub enabled: bool,
    #[serde(with = "duration_serde")]
    pub lease_duration: Duration,
    #[serde(with = "duration_serde")]
    pub renew_deadline: Duration,
    #[serde(with = "duration_serde")]
    pub retry_period: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
    pub ca_file: Option<String>,
    pub verify_client: bool,
    pub min_version: Option<String>,
    pub cipher_suites: Vec<String>,
    pub alpn_protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub tracing: TracingConfig,
    pub metrics: MetricsConfig,
    pub logging: LoggingConfig,
    pub health: HealthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub provider: TracingProvider,
    pub sampling_ratio: f64,
    pub service_name: String,
    pub environment: String,
    pub tags: HashMap<String, String>,
    pub exporters: Vec<TracingExporter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TracingProvider {
    OpenTelemetry,
    Jaeger,
    Zipkin,
    DataDog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingExporter {
    pub name: String,
    pub endpoint: String,
    pub protocol: String,
    pub headers: HashMap<String, String>,
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    pub batch_size: usize,
    #[serde(with = "duration_serde")]
    pub flush_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub enabled: bool,
    pub path: String,
    pub include_details: bool,
    pub checks: Vec<HealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub enabled: bool,
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    #[serde(with = "duration_serde")]
    pub interval: Duration,
    pub initial_delay: Option<Duration>,
    pub required: bool,
}

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_max_request_size() -> usize {
    1024 * 1024 * 10 // 10MB
}

fn default_gateway_protocol() -> GatewayProtocol {
    GatewayProtocol::Rest
}

fn default_backend_protocol() -> BackendProtocol {
    BackendProtocol::Rest
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = std::env::var("GATEWAY_CONFIG")
            .unwrap_or_else(|_| "gateway-config.json".to_string());

        let config = loader::load_config(&std::path::PathBuf::from(config_path))?;
        validator::validate_config(&config)?;
        Ok(config)
    }

    pub fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                ws_port: None,
                workers: num_cpus::get(),
                timeout: default_timeout(),
                max_request_size: default_max_request_size(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file_output: None,
                json_fields: vec![],
            },
            metrics: MetricsConfig {
                enabled: true,
                port: 9090,
                path: "/metrics".to_string(),
                tags: HashMap::new(),
            },
            security: SecurityConfig {
                cors: CorsConfig::default(),
                rate_limit: RateLimitConfig::default(),
                auth: AuthConfig::default(),
                waf: WafConfig::default(),
                rbac: RbacConfig::default(),
            },
            plugins: PluginsConfig {
                enabled: false,
                directory: None,
                wasm_enabled: false,
            },
            endpoints: vec![],
            cluster: ClusterConfig {
                enabled: false,
                discovery_method: None,
                discovery_endpoints: vec![],
                node_name: None,
                node_role: None,
                sync_interval: None,
                leader_election: None,
            },
            tls: TlsConfig {
                enabled: false,
                cert_file: None,
                key_file: None,
                ca_file: None,
                verify_client: false,
                min_version: Some("TLS1.3".to_string()),
                cipher_suites: vec![],
                alpn_protocols: vec![],
            },
            observability: ObservabilityConfig {
                tracing: TracingConfig {
                    enabled: false,
                    provider: TracingProvider::OpenTelemetry,
                    sampling_ratio: 0.5,
                    service_name: "gateway".to_string(),
                    environment: "development".to_string(),
                    tags: HashMap::new(),
                    exporters: vec![],
                },
                metrics: MetricsConfig {
                    enabled: true,
                    port: 9090,
                    path: "/metrics".to_string(),
                    tags: HashMap::new(),
                },
                logging: LoggingConfig {
                    level: "info".to_string(),
                    format: "json".to_string(),
                    file_output: None,
                    json_fields: vec![],
                },
                health: HealthConfig {
                    enabled: false,
                    path: "/health".to_string(),
                    include_details: false,
                    checks: vec![],
                },
            },
        }
    }
} 