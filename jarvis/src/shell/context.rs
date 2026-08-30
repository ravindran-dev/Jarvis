#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    pub last_target: Option<String>,
}

impl SessionContext {
    pub fn resolve_target<'a>(&'a self, target: &'a str) -> Option<&'a str> {
        match target.to_lowercase().as_str() {
            "it" | "that" | "that process" | "the previous one" => self.last_target.as_deref(),
            _ => Some(target),
        }
    }
}
