use sysinfo::System;

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub user: String,
    pub name: String,
    pub cpu_usage: f32,
    pub mem_bytes: u64,
    pub mem_usage_percent: f32,
    pub state: String,
    pub cmd: String,
    pub uptime_secs: u64,
    pub threads: usize,
}

pub struct ProcessTracker {
    sys: System,
}

impl ProcessTracker {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn refresh(&mut self) {
        self.sys
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    }

    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        let total_mem = self.sys.total_memory() as f32;
        let mut processes = Vec::new();

        for (pid, process) in self.sys.processes() {
            let user = match process.user_id() {
                Some(uid) => {
                    // Try to get username if possible, otherwise uid string
                    // Actually, sysinfo has `Users` but we might need to refresh it.
                    // For simplicity:
                    uid.to_string()
                }
                None => "root".to_string(),
            };

            let ppid = process.parent().map(|p| p.as_u32());
            let mem_usage_percent = if total_mem > 0.0 {
                (process.memory() as f32 / total_mem) * 100.0
            } else {
                0.0
            };

            let cmd = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(" ");
            let state = process.status().to_string();

            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                ppid,
                user,
                name: process.name().to_string_lossy().into_owned(),
                cpu_usage: process.cpu_usage(),
                mem_bytes: process.memory(),
                mem_usage_percent,
                state,
                cmd,
                uptime_secs: process.run_time(),
                threads: 1, // sysinfo doesn't easily expose thread count on Linux via standard Process object
            });
        }

        processes
    }
}
