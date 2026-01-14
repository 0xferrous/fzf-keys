use crate::keybind::{Keybind, Modifier};
use crate::source::Source;
use kdl::KdlDocument;
use std::fs;
use std::path::PathBuf;

pub struct NiriSource {
    config_path: PathBuf,
}

impl NiriSource {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn from_default_config() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config/niri/config.kdl")
        } else {
            return Err("HOME environment variable not set".into());
        };

        Ok(Self::new(config_path))
    }

    fn format_action(action_node: &kdl::KdlNode) -> String {
        let action_name = action_node.name().to_string();

        // Collect arguments (entries without names) and properties (entries with names)
        let mut args = Vec::new();
        let mut props = Vec::new();

        for entry in action_node.entries() {
            if entry.name().is_none() {
                // This is an argument
                args.push(Self::format_value(entry.value()));
            } else {
                // This is a property (like skip-confirmation=true)
                if let Some(name) = entry.name() {
                    let value = Self::format_value(entry.value());
                    props.push(format!("{}={}", name.value(), value));
                }
            }
        }

        // Build the action string
        let mut result = action_name;

        if !args.is_empty() {
            result.push(' ');
            result.push_str(&args.join(" "));
        }

        if !props.is_empty() {
            result.push(' ');
            result.push_str(&props.join(" "));
        }

        result
    }

    fn format_value(value: &kdl::KdlValue) -> String {
        if let Some(s) = value.as_string() {
            format!("\"{}\"", s)
        } else if let Some(i) = value.as_integer() {
            i.to_string()
        } else if let Some(b) = value.as_bool() {
            b.to_string()
        } else if let Some(f) = value.as_float() {
            f.to_string()
        } else {
            "null".to_string()
        }
    }

    fn parse_keybind_node(
        &self,
        node: &kdl::KdlNode,
    ) -> Result<Keybind, Box<dyn std::error::Error>> {
        let name = node.name().to_string();

        let (modifiers, key) = Self::parse_key_combination(&name)?;

        // Extract properties from entries
        let mut description = None;
        let mut repeat = None;
        let mut cooldown_ms = None;
        let mut allow_when_locked = None;
        let mut allow_inhibiting = None;

        for entry in node.entries() {
            if let Some(entry_name) = entry.name() {
                match entry_name.value() {
                    "hotkey-overlay-title" => {
                        description = entry.value().as_string().map(|s| s.to_string());
                    }
                    "repeat" => {
                        repeat = entry.value().as_bool();
                    }
                    "cooldown-ms" => {
                        cooldown_ms = entry.value().as_integer().and_then(|v| {
                            if v >= 0 && v <= u64::MAX as i128 {
                                Some(v as u64)
                            } else {
                                None
                            }
                        });
                    }
                    "allow-when-locked" => {
                        allow_when_locked = entry.value().as_bool();
                    }
                    "allow-inhibiting" => {
                        allow_inhibiting = entry.value().as_bool();
                    }
                    _ => {}
                }
            }
        }

        let action = if let Some(children) = node.children() {
            children
                .nodes()
                .iter()
                .map(Self::format_action)
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            "unknown".to_string()
        };

        Ok(Keybind {
            modifiers,
            key,
            action,
            description,
            program: "niri".to_string(),
            repeat,
            cooldown_ms,
            allow_when_locked,
            allow_inhibiting,
        })
    }

    fn parse_key_combination(
        combo: &str,
    ) -> Result<(Vec<Modifier>, String), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = combo.split('+').collect();

        if parts.is_empty() {
            return Err("Empty key combination".into());
        }

        let mut modifiers = Vec::new();
        let key = parts[parts.len() - 1].to_string();

        for part in &parts[..parts.len() - 1] {
            let modifier = match *part {
                "Mod" => Modifier::Mod,
                "Super" | "Win" => Modifier::Super,
                "Alt" => Modifier::Alt,
                "Ctrl" | "Control" => Modifier::Ctrl,
                "Shift" => Modifier::Shift,
                "ISO_Level3_Shift" | "Mod5" => Modifier::IsoLevel3Shift,
                "ISO_Level5_Shift" | "Mod3" => Modifier::IsoLevel5Shift,
                _ => return Err(format!("Unknown modifier: {}", part).into()),
            };
            modifiers.push(modifier);
        }

        Ok((modifiers, key))
    }

    fn parse_config(&self, content: &str) -> Result<Vec<Keybind>, Box<dyn std::error::Error>> {
        let doc: KdlDocument = content.parse()?;

        let mut keybinds = Vec::new();

        for node in doc.nodes() {
            if node.name().to_string() == "binds"
                && let Some(children) = node.children()
            {
                for bind_node in children.nodes() {
                    let name = bind_node.name().to_string();

                    if (name.contains('+') || !name.chars().next().unwrap_or(' ').is_lowercase())
                        && let Ok(keybind) = self.parse_keybind_node(bind_node)
                    {
                        keybinds.push(keybind);
                    }
                }
            }
        }

        Ok(keybinds)
    }
}

impl Source for NiriSource {
    type Item = Keybind;

    fn name(&self) -> &str {
        "niri"
    }

    fn discover(&self) -> Result<Vec<Self::Item>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&self.config_path)?;
        self.parse_config(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_combination() {
        let (mods, key) = NiriSource::parse_key_combination("Mod+Shift+T").unwrap();
        assert_eq!(mods, vec![Modifier::Mod, Modifier::Shift]);
        assert_eq!(key, "T");
    }

    #[test]
    fn test_parse_key_combination_no_modifiers() {
        let (mods, key) = NiriSource::parse_key_combination("XF86AudioRaiseVolume").unwrap();
        assert_eq!(mods, vec![]);
        assert_eq!(key, "XF86AudioRaiseVolume");
    }

    #[test]
    fn test_parse_key_combination_multiple_modifiers() {
        let (mods, key) = NiriSource::parse_key_combination("Mod+Shift+Ctrl+L").unwrap();
        assert_eq!(mods, vec![Modifier::Mod, Modifier::Shift, Modifier::Ctrl]);
        assert_eq!(key, "L");
    }
}
