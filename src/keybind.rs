use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Modifier {
    Mod,
    Super,
    Alt,
    Ctrl,
    Shift,
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Mod => write!(f, "Mod"),
            Modifier::Super => write!(f, "Super"),
            Modifier::Alt => write!(f, "Alt"),
            Modifier::Ctrl => write!(f, "Ctrl"),
            Modifier::Shift => write!(f, "Shift"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keybind {
    pub modifiers: Vec<Modifier>,
    pub key: String,
    pub action: String,
    pub description: Option<String>,
    pub program: String,
}

impl fmt::Display for Keybind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.modifiers.is_empty() {
            for (i, modifier) in self.modifiers.iter().enumerate() {
                if i > 0 {
                    write!(f, "+")?;
                }
                write!(f, "{}", modifier)?;
            }
            write!(f, "+")?;
        }
        write!(f, "{}", self.key)?;

        if let Some(desc) = &self.description {
            write!(f, " - {}", desc)?;
        } else {
            write!(f, " - {}", self.action)?;
        }

        write!(f, " [{}]", self.program)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybind_display() {
        let keybind = Keybind {
            modifiers: vec![Modifier::Mod, Modifier::Shift],
            key: "T".to_string(),
            action: "spawn-terminal".to_string(),
            description: Some("Open Terminal".to_string()),
            program: "niri".to_string(),
        };

        assert_eq!(keybind.to_string(), "Mod+Shift+T - Open Terminal [niri]");
    }

    #[test]
    fn test_keybind_no_description() {
        let keybind = Keybind {
            modifiers: vec![Modifier::Mod],
            key: "Q".to_string(),
            action: "close-window".to_string(),
            description: None,
            program: "niri".to_string(),
        };

        assert_eq!(keybind.to_string(), "Mod+Q - close-window [niri]");
    }

    #[test]
    fn test_keybind_no_modifiers() {
        let keybind = Keybind {
            modifiers: vec![],
            key: "XF86AudioRaiseVolume".to_string(),
            action: "volume-up".to_string(),
            description: Some("Increase Volume".to_string()),
            program: "niri".to_string(),
        };

        assert_eq!(
            keybind.to_string(),
            "XF86AudioRaiseVolume - Increase Volume [niri]"
        );
    }
}
