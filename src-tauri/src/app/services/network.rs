use std::collections::BTreeSet;
use std::process::Command;

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum NetworkReadiness {
    Safe,
    Caution,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum NetworkConflictLevel {
    Info,
    Warning,
    Blocking,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProcessSignal {
    pub pid: u32,
    pub label: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPortBinding {
    pub port: u16,
    pub protocol: String,
    pub process: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDefaultRoute {
    pub interface: Option<String>,
    pub gateway: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceSummary {
    pub name: String,
    pub kind: String,
    pub addresses: Vec<String>,
    pub is_up: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDnsResolver {
    pub scope: String,
    pub resolvers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProxyStatus {
    pub kind: String,
    pub enabled: bool,
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConflict {
    pub level: NetworkConflictLevel,
    pub code: String,
    pub message: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDiagnostics {
    pub default_route_raw: String,
    pub proxy_raw: String,
    pub dns_raw: String,
    pub ifconfig_raw: String,
    pub listen_raw: String,
    pub process_raw: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LocalNetworkSnapshot {
    pub readiness: NetworkReadiness,
    pub headline: String,
    pub reasons: Vec<String>,
    pub default_route: NetworkDefaultRoute,
    pub interfaces: Vec<NetworkInterfaceSummary>,
    pub dns_resolvers: Vec<NetworkDnsResolver>,
    pub proxies: Vec<NetworkProxyStatus>,
    pub default_interface: Option<String>,
    pub default_gateway: Option<String>,
    pub active_interfaces: Vec<String>,
    pub utun_interfaces: Vec<String>,
    pub resolvers: Vec<String>,
    pub system_proxy_enabled: bool,
    pub system_proxy_summary: Vec<String>,
    pub related_processes: Vec<NetworkProcessSignal>,
    pub port_bindings: Vec<NetworkPortBinding>,
    pub conflicts: Vec<NetworkConflict>,
    pub diagnostics: NetworkDiagnostics,
}

#[derive(Debug, Default)]
pub struct NetworkService;

impl NetworkService {
    pub fn new() -> Self {
        Self
    }

    pub fn snapshot(&self) -> LocalNetworkSnapshot {
        let default_route_raw = run_command("route", &["-n", "get", "default"]);
        let proxy_raw = run_command("scutil", &["--proxy"]);
        let dns_raw = run_command("scutil", &["--dns"]);
        let ifconfig_raw = run_command("ifconfig", &[]);
        let listen_raw = run_command("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN"]);
        let process_raw = run_command("ps", &["ax", "-o", "pid=,command="]);

        let (default_interface, default_gateway) = parse_default_route(&default_route_raw);
        let interfaces = parse_interface_summaries(&ifconfig_raw);
        let active_interfaces = interfaces
            .iter()
            .filter(|item| item.is_active)
            .map(|item| item.name.clone())
            .collect::<Vec<_>>();
        let utun_interfaces = active_interfaces
            .iter()
            .filter(|name| name.starts_with("utun"))
            .cloned()
            .collect::<Vec<_>>();
        let dns_resolvers = parse_dns_resolvers(&dns_raw);
        let resolvers = dns_resolvers
            .iter()
            .flat_map(|item| item.resolvers.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let proxies = parse_proxy_statuses(&proxy_raw);
        let system_proxy_summary = summarize_proxies(&proxies);
        let system_proxy_enabled = proxies.iter().any(|item| item.enabled);
        let related_processes = parse_related_processes(&process_raw);
        let port_bindings = parse_port_bindings(&listen_raw);

        let mut conflicts = Vec::new();
        if related_processes
            .iter()
            .any(|process| process.label.contains("Clash Verge"))
            && (system_proxy_enabled || !utun_interfaces.is_empty())
        {
            conflicts.push(NetworkConflict {
                level: NetworkConflictLevel::Blocking,
                code: "clash-verge-active".to_string(),
                message: "Clash Verge appears to be actively managing local network traffic."
                    .to_string(),
                evidence: collect_clash_evidence(
                    &related_processes,
                    &utun_interfaces,
                    &system_proxy_summary,
                ),
            });
        }

        if !utun_interfaces.is_empty() && conflicts.is_empty() {
            conflicts.push(NetworkConflict {
                level: NetworkConflictLevel::Warning,
                code: "tun-present".to_string(),
                message: "Detected active utun interfaces; verify ownership before enabling sing-box TUN.".to_string(),
                evidence: utun_interfaces.clone(),
            });
        }

        if system_proxy_enabled
            && conflicts
                .iter()
                .all(|item| !matches!(item.level, NetworkConflictLevel::Blocking))
        {
            conflicts.push(NetworkConflict {
                level: NetworkConflictLevel::Warning,
                code: "system-proxy-enabled".to_string(),
                message: "System proxy settings are enabled and may already be managed by another client.".to_string(),
                evidence: system_proxy_summary.clone(),
            });
        }

        if !port_bindings.is_empty()
            && conflicts
                .iter()
                .all(|item| !matches!(item.level, NetworkConflictLevel::Blocking))
        {
            conflicts.push(NetworkConflict {
                level: NetworkConflictLevel::Info,
                code: "watched-ports-occupied".to_string(),
                message: "Observed local proxy-related listening ports in use.".to_string(),
                evidence: port_bindings
                    .iter()
                    .map(|binding| binding.detail.clone())
                    .collect(),
            });
        }

        let (readiness, headline) = if conflicts
            .iter()
            .any(|item| matches!(item.level, NetworkConflictLevel::Blocking))
        {
            (
                NetworkReadiness::Blocked,
                "Existing local network ownership detected; keep sing-box TUN disabled for now."
                    .to_string(),
            )
        } else if !conflicts.is_empty() || !related_processes.is_empty() {
            (
                NetworkReadiness::Caution,
                "Local network environment needs verification before a TUN rollout.".to_string(),
            )
        } else if default_interface.is_some() {
            (
                NetworkReadiness::Safe,
                "No strong ownership conflicts detected from the current local network snapshot."
                    .to_string(),
            )
        } else {
            (
                NetworkReadiness::Unknown,
                "Could not resolve enough local network state to make a safe TUN recommendation."
                    .to_string(),
            )
        };

        let mut reasons = Vec::new();
        if let Some(interface) = &default_interface {
            reasons.push(format!(
                "Default route currently resolves through {interface}."
            ));
        }
        if !utun_interfaces.is_empty() {
            reasons.push(format!(
                "Detected utun interfaces: {}.",
                utun_interfaces.join(", ")
            ));
        }
        if system_proxy_enabled {
            reasons.push("macOS system proxy settings are enabled.".to_string());
        }
        if let Some(process) = related_processes.first() {
            reasons.push(format!(
                "Observed related process: {} (pid {}).",
                process.label, process.pid
            ));
        }
        if reasons.is_empty() {
            reasons.push(
                "No obvious proxy/TUN ownership signals were detected in the first pass."
                    .to_string(),
            );
        }

        LocalNetworkSnapshot {
            readiness,
            headline,
            reasons,
            default_route: NetworkDefaultRoute {
                interface: default_interface.clone(),
                gateway: default_gateway.clone(),
            },
            interfaces,
            dns_resolvers,
            proxies,
            default_interface,
            default_gateway,
            active_interfaces,
            utun_interfaces,
            resolvers,
            system_proxy_enabled,
            system_proxy_summary,
            related_processes,
            port_bindings,
            conflicts,
            diagnostics: NetworkDiagnostics {
                default_route_raw,
                proxy_raw,
                dns_raw,
                ifconfig_raw,
                listen_raw,
                process_raw,
            },
        }
    }
}

fn run_command(command: &str, args: &[&str]) -> String {
    match Command::new(command).args(args).output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() {
                format!("command exited with status {}", output.status)
            } else {
                stderr
            }
        }
        Err(err) => format!("failed to run {command}: {err}"),
    }
}

fn parse_default_route(raw: &str) -> (Option<String>, Option<String>) {
    let interface = raw.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("interface:")
            .map(|value| value.trim().to_string())
    });
    let gateway = raw.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("gateway:")
            .map(|value| value.trim().to_string())
    });
    (interface, gateway)
}

fn parse_interface_summaries(raw: &str) -> Vec<NetworkInterfaceSummary> {
    let mut items = Vec::new();
    let mut current: Option<NetworkInterfaceSummary> = None;

    for line in raw.lines() {
        if !line.starts_with('\t') && !line.starts_with(' ') {
            if let Some(summary) = current.take() {
                items.push(summary);
            }

            let Some((name, rest)) = line.split_once(':') else {
                continue;
            };
            let flags = rest.to_lowercase();
            let trimmed = name.trim().to_string();
            current = Some(NetworkInterfaceSummary {
                kind: if trimmed.starts_with("utun") {
                    "utun".to_string()
                } else {
                    "physicalOrVirtual".to_string()
                },
                is_up: flags.contains("up"),
                is_active: flags.contains("running") || flags.contains("up"),
                name: trimmed,
                addresses: Vec::new(),
            });
            continue;
        }

        let Some(summary) = current.as_mut() else {
            continue;
        };
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("inet ") {
            if let Some(address) = value.split_whitespace().next() {
                summary.addresses.push(address.to_string());
            }
        } else if let Some(value) = trimmed.strip_prefix("inet6 ") {
            if let Some(address) = value.split_whitespace().next() {
                summary.addresses.push(address.to_string());
            }
        }
    }

    if let Some(summary) = current {
        items.push(summary);
    }

    items
}

fn parse_dns_resolvers(raw: &str) -> Vec<NetworkDnsResolver> {
    let mut items = Vec::new();
    let mut current_scope = String::from("global");
    let mut current_resolvers = BTreeSet::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("resolver #") {
            if !current_resolvers.is_empty() {
                items.push(NetworkDnsResolver {
                    scope: current_scope.clone(),
                    resolvers: current_resolvers.into_iter().collect(),
                });
                current_resolvers = BTreeSet::new();
            }
            current_scope = trimmed.to_string();
            continue;
        }

        if let Some((_, value)) = trimmed.split_once(':') {
            if trimmed.starts_with("nameserver[") {
                current_resolvers.insert(value.trim().to_string());
            }
        }
    }

    if !current_resolvers.is_empty() {
        items.push(NetworkDnsResolver {
            scope: current_scope,
            resolvers: current_resolvers.into_iter().collect(),
        });
    }

    items
}

fn parse_proxy_statuses(raw: &str) -> Vec<NetworkProxyStatus> {
    ["HTTP", "HTTPS", "SOCKS"]
        .into_iter()
        .map(|kind| NetworkProxyStatus {
            kind: kind.to_string(),
            enabled: raw
                .lines()
                .any(|line| line.trim() == format!("{kind}Enable : 1")),
            host: raw.lines().find_map(|line| {
                line.trim()
                    .strip_prefix(&format!("{kind}Proxy : "))
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            }),
            port: raw.lines().find_map(|line| {
                line.trim()
                    .strip_prefix(&format!("{kind}Port : "))
                    .and_then(|value| value.trim().parse::<u16>().ok())
            }),
        })
        .collect()
}

fn summarize_proxies(proxies: &[NetworkProxyStatus]) -> Vec<String> {
    proxies
        .iter()
        .filter(|item| item.enabled)
        .map(|item| match (&item.host, item.port) {
            (Some(host), Some(port)) => format!("{} proxy enabled at {}:{}", item.kind, host, port),
            _ => format!("{} proxy enabled", item.kind),
        })
        .collect()
}

fn parse_related_processes(raw: &str) -> Vec<NetworkProcessSignal> {
    let keywords = [
        ("clash verge", "Clash Verge"),
        ("clash-verge", "Clash Verge"),
        ("mihomo", "Mihomo"),
        (" clash", "Clash"),
        ("sing-box", "sing-box"),
    ];
    let mut items = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(pid_text) = parts.next() else {
            continue;
        };
        let Ok(pid) = pid_text.parse::<u32>() else {
            continue;
        };
        let command = parts.collect::<Vec<_>>().join(" ");
        let lower = format!(" {}", command.to_lowercase());
        let Some((_, label)) = keywords.iter().find(|(needle, _)| lower.contains(needle)) else {
            continue;
        };
        items.push(NetworkProcessSignal {
            pid,
            label: (*label).to_string(),
            command,
        });
    }
    items
}

fn parse_port_bindings(raw: &str) -> Vec<NetworkPortBinding> {
    let watched_ports = [1080_u16, 7890, 7891, 7892, 7893, 7895, 7897, 9090, 9091];
    let mut items = Vec::new();
    for line in raw.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let columns = trimmed.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 9 {
            continue;
        }
        let process = columns[0].to_string();
        let detail = trimmed.to_string();
        let Some(name_column) = columns.last() else {
            continue;
        };
        let Some(port_text) = name_column.rsplit(':').next() else {
            continue;
        };
        let Ok(port) = port_text.parse::<u16>() else {
            continue;
        };
        if !watched_ports.contains(&port) {
            continue;
        }
        items.push(NetworkPortBinding {
            port,
            protocol: "tcp".to_string(),
            process,
            detail,
        });
    }
    items
}

fn collect_clash_evidence(
    related_processes: &[NetworkProcessSignal],
    utun_interfaces: &[String],
    system_proxy_summary: &[String],
) -> Vec<String> {
    let mut evidence = Vec::new();
    for process in related_processes {
        if process.label.contains("Clash Verge") {
            evidence.push(format!("{} (pid {})", process.command, process.pid));
        }
    }
    evidence.extend(utun_interfaces.iter().cloned());
    evidence.extend(system_proxy_summary.iter().cloned());
    evidence
}
