use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Modifier {
    Mod,
    Super,
    Alt,
    Ctrl,
    Shift,
    IsoLevel3Shift,
    IsoLevel5Shift,
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Mod => write!(f, "Mod"),
            Modifier::Super => write!(f, "Super"),
            Modifier::Alt => write!(f, "Alt"),
            Modifier::Ctrl => write!(f, "Ctrl"),
            Modifier::Shift => write!(f, "Shift"),
            Modifier::IsoLevel3Shift => write!(f, "ISO_Level3_Shift"),
            Modifier::IsoLevel5Shift => write!(f, "ISO_Level5_Shift"),
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
    pub repeat: Option<bool>,
    pub cooldown_ms: Option<u64>,
    pub allow_when_locked: Option<bool>,
    pub allow_inhibiting: Option<bool>,
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

        // Add property annotations if present
        let mut props = Vec::new();
        if let Some(false) = self.repeat {
            props.push("no-repeat".to_string());
        }
        if let Some(cooldown) = self.cooldown_ms {
            props.push(format!("cooldown={}ms", cooldown));
        }
        if let Some(true) = self.allow_when_locked {
            props.push("allow-locked".to_string());
        }
        if let Some(false) = self.allow_inhibiting {
            props.push("no-inhibit".to_string());
        }

        if !props.is_empty() {
            write!(f, " ({})", props.join(", "))?;
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
            repeat: None,
            cooldown_ms: None,
            allow_when_locked: None,
            allow_inhibiting: None,
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
            repeat: None,
            cooldown_ms: None,
            allow_when_locked: None,
            allow_inhibiting: None,
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
            repeat: None,
            cooldown_ms: None,
            allow_when_locked: None,
            allow_inhibiting: None,
        };

        assert_eq!(
            keybind.to_string(),
            "XF86AudioRaiseVolume - Increase Volume [niri]"
        );
    }

    #[test]
    fn test_keybind_with_properties() {
        let keybind = Keybind {
            modifiers: vec![Modifier::Mod],
            key: "WheelScrollDown".to_string(),
            action: "focus-workspace-down".to_string(),
            description: None,
            program: "niri".to_string(),
            repeat: Some(false),
            cooldown_ms: Some(150),
            allow_when_locked: None,
            allow_inhibiting: None,
        };

        assert_eq!(
            keybind.to_string(),
            "Mod+WheelScrollDown - focus-workspace-down (no-repeat, cooldown=150ms) [niri]"
        );
    }

    #[test]
    fn test_keybind_allow_when_locked() {
        let keybind = Keybind {
            modifiers: vec![],
            key: "XF86AudioRaiseVolume".to_string(),
            action: "spawn-sh".to_string(),
            description: Some("Volume Up".to_string()),
            program: "niri".to_string(),
            repeat: None,
            cooldown_ms: None,
            allow_when_locked: Some(true),
            allow_inhibiting: None,
        };

        assert_eq!(
            keybind.to_string(),
            "XF86AudioRaiseVolume - Volume Up (allow-locked) [niri]"
        );
    }
}
