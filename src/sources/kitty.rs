use crate::keybind::{Keybind, Modifier};
use crate::source::Source;
use pyo3::prelude::*;

/// Source for discovering keybinds from kitty terminal.
///
/// # Discovery Method
///
/// This source uses kitty's Python API to discover all active keybinds.
/// It directly calls kitty's `debug_config()` function via PyO3, which returns
/// the current kitty configuration including all keyboard shortcuts:
/// - Default keybinds that are still active
/// - Custom keybinds added in config
/// - Changed keybinds (defaults that were remapped)
/// - Shows the actual resolved keybinds after kitty_mod expansion
///
/// The output includes sections like "Added shortcuts", "Removed shortcuts", and
/// "Changed shortcuts", showing the difference between default config and user config.
///
/// This approach ensures we capture the actual runtime keybinds as kitty sees them,
/// rather than trying to replicate kitty's config parsing logic.
///
/// # Requirements
///
/// - Python with kitty installed must be available
/// - The kitty Python modules must be importable
pub struct KittySource;

impl KittySource {
    pub fn new() -> Self {
        Self
    }

    fn get_keybinds_from_python() -> Result<Vec<Keybind>, Box<dyn std::error::Error>> {
        Python::with_gil(|py| {
            // Import kitty modules
            let kitty_config = py.import_bound("kitty.config").map_err(|e| {
                format!(
                    "Failed to import kitty.config. Is kitty installed? Error: {}",
                    e
                )
            })?;
            let kitty_types = py
                .import_bound("kitty.types")
                .map_err(|e| format!("Failed to import kitty.types: {}", e))?;

            // Load kitty configuration
            let load_config_fn = kitty_config.getattr("load_config")?;
            let opts = load_config_fn.call0()?;

            // Get kitty_mod value
            let kitty_mod: i32 = opts.getattr("kitty_mod")?.extract()?;

            // Get the expanded kitty_mod names (e.g., "ctrl+shift")
            let mod_to_names_fn = kitty_types.getattr("mod_to_names")?;
            let kitty_mod_names_gen = mod_to_names_fn.call1((kitty_mod,))?;
            // mod_to_names returns a generator, convert to list
            let list_builtin = py.eval_bound("list", None, None)?;
            let kitty_mod_names_list = list_builtin.call1((kitty_mod_names_gen,))?;
            let kitty_mod_names_vec: Vec<String> = kitty_mod_names_list.extract()?;
            let kitty_mod_expanded = kitty_mod_names_vec.join("+");

            // Get keyboard_modes
            let keyboard_modes = opts.getattr("keyboard_modes")?;

            // Get the Shortcut class
            let shortcut_class = kitty_types.getattr("Shortcut")?;

            let mut keybinds = Vec::new();

            // Iterate through keyboard modes
            let modes_items = keyboard_modes.call_method0("items")?;
            for mode_item in modes_items.iter()? {
                let mode_item = mode_item?;

                // Extract mode object using getitem
                let mode_obj = mode_item.get_item(1)?;

                // Get the keymap from this mode
                let keymap = mode_obj.getattr("keymap")?;
                let keymap_items = keymap.call_method0("items")?;

                // Iterate through keybinds in this mode
                for item in keymap_items.iter()? {
                    let item = item?;

                    // Extract key and actions using getitem
                    let key = item.get_item(0)?;
                    let actions = item.get_item(1)?;

                    // actions is a list of KeyDefinition objects
                    // Each action might have a different complete key sequence (for multi-key bindings)
                    let actions_len: usize = actions.len()?;
                    if actions_len == 0 {
                        continue;
                    }

                    // Process each action separately, as they may have different key sequences
                    for i in 0..actions_len {
                        let action = actions.get_item(i)?;

                        // Create Shortcut from the action's key sequence
                        let is_sequence: bool = action.getattr("is_sequence")?.extract()?;

                        let shortcut = if is_sequence {
                            // For sequences: Shortcut((trigger,) + rest)
                            let trigger = action.getattr("trigger")?;
                            let rest = action.getattr("rest")?;

                            // Use Python to concatenate tuples: (trigger,) + rest
                            let trigger_tuple = pyo3::types::PyTuple::new_bound(py, vec![trigger]);
                            let keys_tuple = trigger_tuple.call_method1("__add__", (rest,))?;

                            // Call Shortcut with the tuple
                            shortcut_class.call1((keys_tuple,))?
                        } else {
                            // For non-sequences: Shortcut((key,))
                            let keys_tuple = pyo3::types::PyTuple::new_bound(py, vec![key.clone()]);

                            // Call Shortcut with the tuple
                            shortcut_class.call1((keys_tuple,))?
                        };

                        // Get human-readable key representation
                        let key_repr: String = shortcut
                            .call_method1("human_repr", (kitty_mod,))?
                            .extract()?;

                        // Replace "kitty_mod" with the actual expanded modifiers
                        let key_repr = key_repr.replace("kitty_mod", &kitty_mod_expanded);

                        // Get action string
                        let action_str: String = action.call_method0("human_repr")?.extract()?;

                        // Parse the key combination
                        let (modifiers, key_name) = Self::parse_key_combination(&key_repr)
                            .map_err(|e| format!("Failed to parse key '{}': {}", key_repr, e))?;

                        keybinds.push(Keybind {
                            modifiers,
                            key: key_name,
                            action: action_str,
                            description: None,
                            program: "kitty".to_string(),
                            repeat: None,
                            cooldown_ms: None,
                            allow_when_locked: None,
                            allow_inhibiting: None,
                        });
                    }
                }
            }

            Ok(keybinds)
        })
    }

    fn parse_key_combination(
        combo: &str,
    ) -> Result<(Vec<Modifier>, String), Box<dyn std::error::Error>> {
        // Special case: if combo ends with "++", the key is "+"
        if combo.ends_with("++") {
            let mod_part = &combo[..combo.len() - 2];
            let mut modifiers = Vec::new();
            if !mod_part.is_empty() {
                for part in mod_part.split('+') {
                    modifiers.push(Self::parse_modifier(part)?);
                }
            }
            return Ok((modifiers, "+".to_string()));
        }

        // Handle multi-key sequences (e.g., "ctrl+f>2")
        if combo.contains('>') {
            // Multi-key sequence: split on '+' before the '>'
            let sequence_parts: Vec<&str> = combo.split('>').collect();
            let first_part = sequence_parts[0];
            let rest = sequence_parts[1..].join(">");

            let parts: Vec<&str> = first_part.split('+').collect();
            if parts.is_empty() {
                return Err("Empty key combination".into());
            }

            let mut modifiers = Vec::new();

            // All parts before the last are modifiers
            for part in &parts[..parts.len() - 1] {
                modifiers.push(Self::parse_modifier(part)?);
            }

            // Last part of first sequence + the rest forms the key
            let key = format!("{}{}{}", parts[parts.len() - 1], ">", rest);

            return Ok((modifiers, key));
        }

        // Normal key combination (e.g., "ctrl+shift+c")
        let parts: Vec<&str> = combo.split('+').collect();

        if parts.is_empty() {
            return Err("Empty key combination".into());
        }

        let mut modifiers = Vec::new();
        let key = parts[parts.len() - 1].to_string();

        // All parts before the last are modifiers
        for part in &parts[..parts.len() - 1] {
            modifiers.push(Self::parse_modifier(part)?);
        }

        Ok((modifiers, key))
    }

    fn parse_modifier(name: &str) -> Result<Modifier, Box<dyn std::error::Error>> {
        match name.to_lowercase().as_str() {
            "ctrl" | "control" => Ok(Modifier::Ctrl),
            "shift" => Ok(Modifier::Shift),
            "alt" | "opt" | "option" => Ok(Modifier::Alt),
            "super" | "cmd" | "command" => Ok(Modifier::Super),
            "kitty_mod" => Ok(Modifier::Mod), // kitty_mod is a configurable modifier
            _ => Err(format!("Unknown modifier: {}", name).into()),
        }
    }
}

impl Source for KittySource {
    type Item = Keybind;

    fn name(&self) -> &str {
        "kitty"
    }

    fn discover(&self) -> Result<Vec<Self::Item>, Box<dyn std::error::Error>> {
        Self::get_keybinds_from_python()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_combination() {
        let (mods, key) = KittySource::parse_key_combination("ctrl+shift+t").unwrap();
        assert_eq!(mods, vec![Modifier::Ctrl, Modifier::Shift]);
        assert_eq!(key, "t");
    }

    #[test]
    fn test_parse_key_combination_no_modifiers() {
        let (mods, key) = KittySource::parse_key_combination("f1").unwrap();
        assert_eq!(mods, vec![]);
        assert_eq!(key, "f1");
    }

    #[test]
    fn test_parse_multi_key_sequence() {
        let (mods, key) = KittySource::parse_key_combination("ctrl+f>2").unwrap();
        assert_eq!(mods, vec![Modifier::Ctrl]);
        assert_eq!(key, "f>2");
    }

    #[test]
    fn test_parse_plus_key() {
        let (mods, key) = KittySource::parse_key_combination("ctrl+shift++").unwrap();
        assert_eq!(mods, vec![Modifier::Ctrl, Modifier::Shift]);
        assert_eq!(key, "+");
    }

    #[test]
    fn test_parse_kitty_mod() {
        let (mods, key) = KittySource::parse_key_combination("kitty_mod+c").unwrap();
        assert_eq!(mods, vec![Modifier::Mod]);
        assert_eq!(key, "c");
    }
}
