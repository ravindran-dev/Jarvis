use std::process::Command;

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub uid: String,
    pub home: String,
    pub shell: String,
}

pub struct UserTracker;

impl UserTracker {
    pub fn new() -> Self {
        Self
    }

    pub fn get_users(&self) -> Vec<UserInfo> {
        let mut users = Vec::new();

        if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 7 {
                    let uid = parts[2].parse::<u32>().unwrap_or(0);
                    // Filter to actual user accounts (typically UID >= 1000 and root)
                    if uid >= 1000 || uid == 0 {
                        users.push(UserInfo {
                            username: parts[0].to_string(),
                            uid: parts[2].to_string(),
                            home: parts[5].to_string(),
                            shell: parts[6].to_string(),
                        });
                    }
                }
            }
        }

        users.sort_by(|a, b| {
            a.uid
                .parse::<u32>()
                .unwrap_or(0)
                .cmp(&b.uid.parse::<u32>().unwrap_or(0))
        });
        users
    }
}
