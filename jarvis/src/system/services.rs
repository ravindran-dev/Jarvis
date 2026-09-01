use std::process::Command;

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub status: String,
    pub enabled: String,
}

pub struct ServiceTracker;

impl ServiceTracker {
    pub fn new() -> Self {
        Self
    }

    pub fn get_services(&self) -> Vec<ServiceInfo> {
        let mut services = Vec::new();

        let output = Command::new("systemctl")
            .arg("list-unit-files")
            .arg("--type=service")
            .arg("--no-pager")
            .arg("--no-legend")
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let enabled = parts[1].to_string(); // "enabled", "disabled", "static"

                    // We can also infer 'status' via systemctl is-active but it's too slow to run per service.
                    // For now, we will query list-units directly.
                    services.push(ServiceInfo {
                        name,
                        status: String::from("Unknown"),
                        enabled,
                    });
                }
            }
        }

        // Fetch running status in bulk
        let active_output = Command::new("systemctl")
            .arg("list-units")
            .arg("--type=service")
            .arg("--all")
            .arg("--no-pager")
            .arg("--no-legend")
            .output();

        if let Ok(active_output) = active_output {
            let stdout = String::from_utf8_lossy(&active_output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let name = parts[0];
                    let active = parts[2]; // active, inactive, failed
                    let sub = parts[3]; // running, dead, exited

                    let display_status = match sub {
                        "running" => "Running",
                        "dead" => "Dead",
                        "exited" => "Exited",
                        _ => active,
                    };

                    if let Some(s) = services.iter_mut().find(|s| s.name == name) {
                        s.status = display_status.to_string();
                    }
                }
            }
        }

        services
    }
}
