pub mod discovery;

use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub use discovery::{discover_endpoints, EndpointSuggestion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub url: String,
    #[serde(default = "default_health_path")]
    pub health_path: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_interval")]
    pub interval_seconds: u32,
}

fn default_health_path() -> String {
    "/health".to_string()
}
fn default_enabled() -> bool {
    true
}
fn default_interval() -> u32 {
    300
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LivenessStatus {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessResult {
    pub environment_name: String,
    pub status: LivenessStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub checked_at: OffsetDateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Global liveness feature config stored in the top-level Config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_interval")]
    pub default_interval_seconds: u32,
    #[serde(default)]
    pub notify_on_failure: bool,
}

impl Default for LivenessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_interval_seconds: 300,
            notify_on_failure: false,
        }
    }
}

/// In-memory cache of liveness results keyed by `(repo_id, environment_name)`.
#[derive(Debug, Default)]
pub struct LivenessCache {
    pub results: HashMap<(Uuid, String), LivenessResult>,
    pub last_probe: HashMap<(Uuid, String), Instant>,
}

impl LivenessCache {
    pub fn store(&mut self, repo_id: Uuid, result: LivenessResult) {
        let key = (repo_id, result.environment_name.clone());
        self.last_probe.insert(key.clone(), Instant::now());
        self.results.insert(key, result);
    }

    pub fn get(&self, repo_id: Uuid, env_name: &str) -> Option<&LivenessResult> {
        self.results.get(&(repo_id, env_name.to_string()))
    }

    pub fn get_all_for_repo(&self, repo_id: Uuid) -> Vec<&LivenessResult> {
        self.results
            .iter()
            .filter(|((rid, _), _)| *rid == repo_id)
            .map(|(_, v)| v)
            .collect()
    }

    pub fn should_probe(&self, repo_id: Uuid, env_name: &str, interval_seconds: u32) -> bool {
        match self.last_probe.get(&(repo_id, env_name.to_string())) {
            Some(last) => last.elapsed().as_secs() >= u64::from(interval_seconds),
            None => true,
        }
    }
}

/// Normalize a probe URL by combining base URL and health path without double
/// slashes at the join point.
pub fn normalize_probe_url(base_url: &str, health_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = if health_path.starts_with('/') {
        health_path.to_string()
    } else {
        format!("/{health_path}")
    };
    format!("{base}{path}")
}

/// Probe an environment using the provided HTTP GET function.
///
/// The `http_get` closure takes a URL and returns `Ok(status_code)` on a
/// successful connection or `Err(message)` on timeout / connection error.
/// This indirection keeps the core testable without a live network.
pub fn probe_environment<F>(env: &Environment, http_get: F) -> LivenessResult
where
    F: FnOnce(&str) -> Result<(u16, u64), String>,
{
    let url = normalize_probe_url(&env.url, &env.health_path);
    let now = OffsetDateTime::now_utc();

    match http_get(&url) {
        Ok((status, elapsed_ms)) => {
            let up = (200..300).contains(&status);
            LivenessResult {
                environment_name: env.name.clone(),
                status: if up {
                    LivenessStatus::Up
                } else {
                    LivenessStatus::Down
                },
                checked_at: now,
                response_time_ms: Some(elapsed_ms),
                error: if up {
                    None
                } else {
                    Some(format!("HTTP {status}"))
                },
            }
        }
        Err(msg) => LivenessResult {
            environment_name: env.name.clone(),
            status: LivenessStatus::Down,
            checked_at: now,
            response_time_ms: None,
            error: Some(msg),
        },
    }
}

/// Perform an actual HTTP GET using `reqwest::blocking` with a 10-second
/// timeout and up to 3 redirects. Returns `(status_code, elapsed_ms)`.
#[cfg(feature = "liveness")]
pub fn reqwest_http_get(url: &str) -> Result<(u16, u64), String> {
    use std::time::Duration;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|e| e.to_string())?;

    let start = Instant::now();
    let resp = client.get(url).send().map_err(|e| {
        if e.is_timeout() {
            "request timed out (10s)".to_string()
        } else if e.is_connect() {
            format!("connection error: {e}")
        } else {
            e.to_string()
        }
    })?;

    let elapsed_ms = start.elapsed().as_millis() as u64;
    Ok((resp.status().as_u16(), elapsed_ms))
}

/// Validate an Environment before persisting. Returns a list of validation
/// errors (empty = valid).
pub fn validate_environment(env: &Environment) -> Vec<String> {
    let mut errors = Vec::new();
    if env.name.trim().is_empty() {
        errors.push("name must not be empty".into());
    }
    let url = env.url.trim().to_lowercase();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        errors.push("url must start with http:// or https://".into());
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_env() -> Environment {
        Environment {
            name: "staging".into(),
            url: "https://staging.example.com".into(),
            health_path: "/health".into(),
            enabled: true,
            interval_seconds: 300,
        }
    }

    #[test]
    fn environment_serde_round_trip() {
        let env = sample_env();
        let json = serde_json::to_string(&env).unwrap();
        let parsed: Environment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "staging");
        assert_eq!(parsed.interval_seconds, 300);
    }

    #[test]
    fn environment_defaults_applied() {
        let json = r#"{"name":"prod","url":"https://prod.example.com"}"#;
        let env: Environment = serde_json::from_str(json).unwrap();
        assert_eq!(env.health_path, "/health");
        assert!(env.enabled);
        assert_eq!(env.interval_seconds, 300);
    }

    #[test]
    fn normalize_probe_url_no_double_slash() {
        assert_eq!(
            normalize_probe_url("https://example.com/", "/health"),
            "https://example.com/health"
        );
        assert_eq!(
            normalize_probe_url("https://example.com", "/health"),
            "https://example.com/health"
        );
        assert_eq!(
            normalize_probe_url("https://example.com/", "health"),
            "https://example.com/health"
        );
    }

    #[test]
    fn probe_up_on_200() {
        let env = sample_env();
        let result = probe_environment(&env, |_url| Ok((200, 42)));
        assert_eq!(result.status, LivenessStatus::Up);
        assert_eq!(result.response_time_ms, Some(42));
        assert!(result.error.is_none());
    }

    #[test]
    fn probe_up_on_204() {
        let env = sample_env();
        let result = probe_environment(&env, |_url| Ok((204, 10)));
        assert_eq!(result.status, LivenessStatus::Up);
    }

    #[test]
    fn probe_down_on_500() {
        let env = sample_env();
        let result = probe_environment(&env, |_url| Ok((500, 100)));
        assert_eq!(result.status, LivenessStatus::Down);
        assert_eq!(result.error.as_deref(), Some("HTTP 500"));
    }

    #[test]
    fn probe_down_on_timeout() {
        let env = sample_env();
        let result = probe_environment(&env, |_url| Err("request timed out".into()));
        assert_eq!(result.status, LivenessStatus::Down);
        assert_eq!(result.error.as_deref(), Some("request timed out"));
        assert!(result.response_time_ms.is_none());
    }

    #[test]
    fn validate_rejects_empty_name() {
        let env = Environment {
            name: "".into(),
            url: "https://example.com".into(),
            ..sample_env()
        };
        let errors = validate_environment(&env);
        assert!(errors.iter().any(|e| e.contains("name")));
    }

    #[test]
    fn validate_rejects_non_http_url() {
        let env = Environment {
            url: "ftp://example.com".into(),
            ..sample_env()
        };
        let errors = validate_environment(&env);
        assert!(errors.iter().any(|e| e.contains("http")));
    }

    #[test]
    fn validate_accepts_valid_env() {
        let env = sample_env();
        assert!(validate_environment(&env).is_empty());
    }

    #[test]
    fn liveness_result_serde_round_trip() {
        let result = LivenessResult {
            environment_name: "staging".into(),
            status: LivenessStatus::Up,
            checked_at: OffsetDateTime::now_utc(),
            response_time_ms: Some(42),
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: LivenessResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, LivenessStatus::Up);
        assert_eq!(parsed.response_time_ms, Some(42));
    }

    #[test]
    fn cache_store_and_retrieve() {
        let mut cache = LivenessCache::default();
        let repo_id = Uuid::new_v4();
        let result = LivenessResult {
            environment_name: "staging".into(),
            status: LivenessStatus::Up,
            checked_at: OffsetDateTime::now_utc(),
            response_time_ms: Some(42),
            error: None,
        };
        cache.store(repo_id, result);
        assert!(cache.get(repo_id, "staging").is_some());
        assert!(cache.get(repo_id, "prod").is_none());
    }

    #[test]
    fn cache_should_probe_initially() {
        let cache = LivenessCache::default();
        assert!(cache.should_probe(Uuid::new_v4(), "staging", 300));
    }

    #[test]
    fn cache_should_not_probe_recently() {
        let mut cache = LivenessCache::default();
        let repo_id = Uuid::new_v4();
        cache.store(
            repo_id,
            LivenessResult {
                environment_name: "staging".into(),
                status: LivenessStatus::Up,
                checked_at: OffsetDateTime::now_utc(),
                response_time_ms: Some(10),
                error: None,
            },
        );
        assert!(!cache.should_probe(repo_id, "staging", 300));
    }

    #[test]
    fn liveness_config_defaults() {
        let config = LivenessConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_interval_seconds, 300);
        assert!(!config.notify_on_failure);
    }
}
