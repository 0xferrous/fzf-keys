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

    fn parse_keybind_node(
        &self,
        node: &kdl::KdlNode,
    ) -> Result<Keybind, Box<dyn std::error::Error>> {
        let name = node.name().to_string();

        let (modifiers, key) = Self::parse_key_combination(&name)?;

        let description = node.entries().iter().find_map(|entry| {
            if let Some(name) = entry.name()
                && name.value() == "hotkey-overlay-title"
            {
                return entry.value().as_string().map(|s| s.to_string());
            }
            None
        });

        let action = if let Some(children) = node.children() {
            children
                .nodes()
                .iter()
                .map(|n| n.name().to_string())
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
                "Super" => Modifier::Super,
                "Alt" => Modifier::Alt,
                "Ctrl" => Modifier::Ctrl,
                "Shift" => Modifier::Shift,
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
    fn name(&self) -> &str {
        "niri"
    }

    fn discover(&self) -> Result<Vec<Keybind>, Box<dyn std::error::Error>> {
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
