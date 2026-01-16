use ratatui::style::Color;

/// Theme configuration
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub background: Color,
    pub text: Color,
}

impl Theme {
    /// Create a default dark theme (cyan)
    pub fn dark() -> Self {
        Self {
            name: "Dark (Cyan)".to_string(),
            primary: Color::Cyan,
            secondary: Color::DarkGray,
            accent: Color::Yellow,
            success: Color::Green,
            warning: Color::Yellow,
            danger: Color::Red,
            background: Color::Black,
            text: Color::White,
        }
    }

    /// Create a blue theme
    pub fn blue() -> Self {
        Self {
            name: "Blue".to_string(),
            primary: Color::Blue,
            secondary: Color::DarkGray,
            accent: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            danger: Color::Red,
            background: Color::Black,
            text: Color::White,
        }
    }

    /// Create a green theme
    pub fn green() -> Self {
        Self {
            name: "Green".to_string(),
            primary: Color::Green,
            secondary: Color::DarkGray,
            accent: Color::Yellow,
            success: Color::LightGreen,
            warning: Color::Yellow,
            danger: Color::Red,
            background: Color::Black,
            text: Color::White,
        }
    }

    /// Create a purple theme
    pub fn purple() -> Self {
        Self {
            name: "Purple".to_string(),
            primary: Color::Magenta,
            secondary: Color::DarkGray,
            accent: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            danger: Color::Red,
            background: Color::Black,
            text: Color::White,
        }
    }

    /// Create a high-contrast theme
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            primary: Color::White,
            secondary: Color::Black,
            accent: Color::Yellow,
            success: Color::LightGreen,
            warning: Color::Yellow,
            danger: Color::LightRed,
            background: Color::Black,
            text: Color::White,
        }
    }

    /// Get all available themes
    pub fn all() -> Vec<Self> {
        vec![
            Self::dark(),
            Self::blue(),
            Self::green(),
            Self::purple(),
            Self::high_contrast(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_themes_available() {
        let themes = Theme::all();
        assert_eq!(themes.len(), 5);
    }
}
