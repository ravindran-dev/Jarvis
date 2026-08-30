use jarvis_core::types::ActionResult;

pub struct ConversationalResponse;

impl ConversationalResponse {
    pub fn generate(result: &ActionResult) -> String {
        match result {
            ActionResult::Success {
                action,
                target,
                details,
                ..
            } => {
                if let Some(t) = target {
                    format!("Done. {} is {}.\n{}", t, action, details)
                } else {
                    format!("Done.\n{}", details)
                }
            }
            ActionResult::Information { data } => data.to_string(),
            ActionResult::NetworkConnections(connections) => {
                let mut out = String::new();
                out.push_str(&format!(
                    "{:<6} {:<20} {:<20} {:<12} {:<10} {}\n",
                    "PROTO", "LOCAL", "REMOTE", "STATE", "PID", "NAME"
                ));
                for conn in connections {
                    let pid = conn
                        .pid
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    let name = conn.process_name.clone().unwrap_or_else(|| "-".to_string());
                    out.push_str(&format!(
                        "{:<6} {:<20} {:<20} {:<12} {:<10} {}\n",
                        conn.protocol, conn.local_addr, conn.remote_addr, conn.state, pid, name
                    ));
                }
                out
            }
            ActionResult::Warning { details } => {
                format!("Warning: {}", details)
            }
            ActionResult::NeedsConfirmation {
                action: _,
                impact,
                warning,
            } => {
                format!(
                    "This will {}. {}. Do you want me to continue? [y/N]",
                    impact, warning
                )
            }
            ActionResult::Failure { reason, error } => {
                if let Some(e) = error {
                    format!("I couldn't do that: {}. {}", reason, e)
                } else {
                    format!("I couldn't do that: {}.", reason)
                }
            }
        }
    }
}
