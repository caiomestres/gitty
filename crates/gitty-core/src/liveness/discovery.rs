use std::path::Path;

use serde::{Deserialize, Serialize};

/// A suggested endpoint discovered from repository files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EndpointSuggestion {
    pub name: String,
    pub url: String,
    pub health_path: String,
    pub source_file: String,
    pub description: String,
}

/// Discover potential health endpoints from repository configuration files.
///
/// Scans the repository root for:
/// - docker-compose.yml / docker-compose.yaml (port mappings)
/// - Dockerfile (EXPOSE instructions)
/// - .env / .env.* files (PORT, SERVER_PORT, APP_PORT variables)
/// - Procfile (process types with port inference)
/// - Kubernetes manifests (*.yaml with Service definitions)
///
/// Returns a list of suggested endpoints with metadata about their source.
pub fn discover_endpoints(repo_path: &Path) -> Vec<EndpointSuggestion> {
    let mut suggestions = Vec::new();
    let mut seen_ports: std::collections::HashSet<u16> = std::collections::HashSet::new();

    let mut add_suggestion = |name: &str, port: u16, source: &str, desc: &str| {
        if seen_ports.insert(port) {
            suggestions.push(EndpointSuggestion {
                name: name.to_string(),
                url: format!("http://localhost:{}", port),
                health_path: "/health".to_string(),
                source_file: source.to_string(),
                description: desc.to_string(),
            });
        }
    };

    for filename in &["docker-compose.yml", "docker-compose.yaml"] {
        let path = repo_path.join(filename);
        if let Ok(content) = std::fs::read_to_string(&path) {
            extract_docker_compose_ports(&content, filename, &mut add_suggestion);
        }
    }

    let dockerfile_path = repo_path.join("Dockerfile");
    if let Ok(content) = std::fs::read_to_string(&dockerfile_path) {
        extract_dockerfile_ports(&content, "Dockerfile", &mut add_suggestion);
    }

    if let Ok(entries) = std::fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(".env") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    extract_env_ports(&content, &name_str, &mut add_suggestion);
                }
            }
        }
    }

    let procfile_path = repo_path.join("Procfile");
    if let Ok(content) = std::fs::read_to_string(&procfile_path) {
        extract_procfile_ports(&content, "Procfile", &mut add_suggestion);
    }

    if let Ok(entries) = std::fs::read_dir(repo_path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".yaml") || name_str.ends_with(".yml") {
                if name_str.contains("docker-compose") {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    extract_k8s_ports(&content, &name_str, &mut add_suggestion);
                }
            }
        }
    }

    suggestions
}

fn extract_docker_compose_ports<F>(content: &str, filename: &str, add: &mut F)
where
    F: FnMut(&str, u16, &str, &str),
{
    use std::sync::LazyLock;
    static PORT_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(\d+):\d+").unwrap());
    static SVC_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^\s{2}(\w[\w-]*):\s*$").unwrap());
    let port_regex = &*PORT_RE;
    let service_regex = &*SVC_RE;

    let mut current_service = String::from("app");

    for line in content.lines() {
        if let Some(cap) = service_regex.captures(line) {
            current_service = cap[1].to_string();
        }

        for cap in port_regex.captures_iter(line) {
            if let Ok(port) = cap[1].parse::<u16>() {
                add(
                    &current_service,
                    port,
                    filename,
                    &format!(
                        "Port {port} exposed in {current_service} service (from docker-compose)"
                    ),
                );
            }
        }
    }
}

fn extract_dockerfile_ports<F>(content: &str, filename: &str, add: &mut F)
where
    F: FnMut(&str, u16, &str, &str),
{
    use std::sync::LazyLock;
    static EXPOSE_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)EXPOSE\s+(\d+)").unwrap());
    let expose_regex = &*EXPOSE_RE;

    for cap in expose_regex.captures_iter(content) {
        if let Ok(port) = cap[1].parse::<u16>() {
            add(
                "docker",
                port,
                filename,
                &format!("Port {} exposed in Dockerfile", port),
            );
        }
    }
}

fn extract_env_ports<F>(content: &str, filename: &str, add: &mut F)
where
    F: FnMut(&str, u16, &str, &str),
{
    use std::sync::LazyLock;
    static ENV_PORT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r"(?i)^(PORT|SERVER_PORT|APP_PORT|HTTP_PORT|API_PORT|WEB_PORT)\s*=\s*(\d+)",
        )
        .unwrap()
    });
    let port_regex = &*ENV_PORT_RE;

    for cap in port_regex.captures_iter(content) {
        let var_name = &cap[1];
        if let Ok(port) = cap[2].parse::<u16>() {
            add(
                "local",
                port,
                filename,
                &format!("Port {} from {} variable", port, var_name),
            );
        }
    }
}

fn extract_procfile_ports<F>(content: &str, filename: &str, add: &mut F)
where
    F: FnMut(&str, u16, &str, &str),
{
    let process_ports: std::collections::HashMap<&str, u16> = [
        ("web", 8080),
        ("api", 3000),
        ("server", 5000),
        ("app", 3000),
        ("worker", 8080),
    ]
    .into_iter()
    .collect();

    for line in content.lines() {
        if let Some((process, _)) = line.split_once(':') {
            let process = process.trim();
            if let Some(&port) = process_ports.get(process) {
                add(
                    process,
                    port,
                    filename,
                    &format!("{} process (typical port for {})", process, process),
                );
            }
        }
    }
}

fn extract_k8s_ports<F>(content: &str, filename: &str, add: &mut F)
where
    F: FnMut(&str, u16, &str, &str),
{
    use std::sync::LazyLock;
    static K8S_PORT_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"port:\s*(\d+)").unwrap());
    static K8S_TARGET_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"targetPort:\s*(\d+)").unwrap());
    let port_regex = &*K8S_PORT_RE;
    let target_regex = &*K8S_TARGET_RE;

    for cap in target_regex.captures_iter(content) {
        if let Ok(port) = cap[1].parse::<u16>() {
            add(
                "k8s-service",
                port,
                filename,
                &format!("Target port {} from Kubernetes Service", port),
            );
        }
    }

    if target_regex.find(content).is_none() {
        for cap in port_regex.captures_iter(content) {
            if let Ok(port) = cap[1].parse::<u16>() {
                add(
                    "k8s-service",
                    port,
                    filename,
                    &format!("Port {} from Kubernetes Service", port),
                );
            }
        }
    }
}
