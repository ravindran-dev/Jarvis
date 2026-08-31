use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonRequest {
    StartService {
        target: String,
        force: bool,
    },
    StopService {
        target: String,
        force: bool,
    },
    RestartService {
        target: String,
        force: bool,
    },
    EnableService {
        target: String,
        force: bool,
    },
    DisableService {
        target: String,
        force: bool,
    },
    ApplyCgroupLimit {
        target: String,
        resource: String,
        value: String,
        force: bool,
    },
    RemoveCgroupLimit {
        target: String,
        force: bool,
    },
    NetworkBlock {
        target: String,
        force: bool,
    },
    NetworkAllow {
        target: String,
        force: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonResponse {
    Success(crate::types::ActionResult),
    Error(String),
}

impl DaemonRequest {
    pub fn from_cmd(action: &str, args: &[&str]) -> Option<Self> {
        let force = args.contains(&"--force");
        let clean_args: Vec<&str> = args.iter().filter(|&&s| s != "--force").copied().collect();

        match action {
            "start" => Some(Self::StartService {
                target: clean_args.first()?.to_string(),
                force,
            }),
            "stop" => Some(Self::StopService {
                target: clean_args.first()?.to_string(),
                force,
            }),
            "restart" => Some(Self::RestartService {
                target: clean_args.first()?.to_string(),
                force,
            }),
            "enable" => Some(Self::EnableService {
                target: clean_args.first()?.to_string(),
                force,
            }),
            "disable" => Some(Self::DisableService {
                target: clean_args.first()?.to_string(),
                force,
            }),
            "limit" => {
                if clean_args.len() < 3 {
                    return None;
                }
                Some(Self::ApplyCgroupLimit {
                    target: clean_args[0].to_string(),
                    resource: clean_args[1].to_string(),
                    value: clean_args[2].to_string(),
                    force,
                })
            }
            "unlimit" => {
                if clean_args.is_empty() {
                    return None;
                }
                Some(Self::RemoveCgroupLimit {
                    target: clean_args[0].to_string(),
                    force,
                })
            }
            "net" => {
                if clean_args.len() < 2 {
                    return None;
                }
                let op = clean_args[0];
                let target = clean_args[1].to_string();
                if op == "block" {
                    Some(Self::NetworkBlock { target, force })
                } else if op == "allow" {
                    Some(Self::NetworkAllow { target, force })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub struct DaemonClient {
    socket_path: String,
}

impl DaemonClient {
    pub fn get_socket_path() -> String {
        if std::path::Path::new("/run/jarvis/jarvis.sock").exists() {
            "/run/jarvis/jarvis.sock".to_string()
        } else if std::path::Path::new("/var/run/jarvis/jarvis.sock").exists() {
            "/var/run/jarvis/jarvis.sock".to_string()
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/.jarvis/jarvis.sock", home)
        }
    }

    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            socket_path: Self::get_socket_path(),
        }
    }

    /// Check if the daemon is running by attempting to connect to the socket
    pub fn is_running(&self) -> bool {
        std::os::unix::net::UnixStream::connect(&self.socket_path).is_ok()
    }

    /// Send a request to the daemon via UNIX socket
    pub fn send_request(&self, req: DaemonRequest) -> Result<DaemonResponse> {
        use std::io::{Read, Write};

        let mut stream = std::os::unix::net::UnixStream::connect(&self.socket_path)?;
        let request_str = serde_json::to_string(&req)?;

        stream.write_all(request_str.as_bytes())?;
        stream.write_all(b"\n")?;

        let mut response_str = String::new();
        stream.read_to_string(&mut response_str)?;

        let response: DaemonResponse = serde_json::from_str(&response_str)?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_request_from_cmd() {
        let req = DaemonRequest::from_cmd("stop", &["jarvis.service", "--force"]).unwrap();
        match req {
            DaemonRequest::StopService { target, force } => {
                assert_eq!(target, "jarvis.service");
                assert!(force);
            }
            _ => panic!("Wrong variant!"),
        }

        let req2 = DaemonRequest::from_cmd("net", &["allow", "1.2.3.4"]).unwrap();
        match req2 {
            DaemonRequest::NetworkAllow { target, force } => {
                assert_eq!(target, "1.2.3.4");
                assert!(!force);
            }
            _ => panic!("Wrong variant!"),
        }
    }
}
