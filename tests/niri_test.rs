use fzf_keys::keybind::Modifier;
use fzf_keys::source::Source;
use fzf_keys::sources::niri::NiriSource;
use std::path::PathBuf;

#[test]
fn test_parse_niri_default_config() {
    let config_path = PathBuf::from("tests/niri-default-config.kdl");
    let source = NiriSource::new(config_path);

    let keybinds = source.discover().expect("Failed to parse config");

    assert!(!keybinds.is_empty(), "Should find keybinds in config");

    println!("Found {} keybinds", keybinds.len());

    for keybind in keybinds.iter().take(5) {
        println!("{}", keybind);
    }
}

#[test]
fn test_niri_specific_keybinds() {
    let config_path = PathBuf::from("tests/niri-default-config.kdl");
    let source = NiriSource::new(config_path);

    let keybinds = source.discover().expect("Failed to parse config");

    let hotkey_overlay = keybinds
        .iter()
        .find(|k| k.key == "Slash" && k.modifiers.contains(&Modifier::Mod));

    assert!(
        hotkey_overlay.is_some(),
        "Should find Mod+Shift+Slash keybind"
    );

    let hotkey = hotkey_overlay.unwrap();
    assert_eq!(hotkey.action, "show-hotkey-overlay");
    assert_eq!(hotkey.program, "niri");
}

#[test]
fn test_niri_keybind_with_description() {
    let config_path = PathBuf::from("tests/niri-default-config.kdl");
    let source = NiriSource::new(config_path);

    let keybinds = source.discover().expect("Failed to parse config");

    let terminal_bind = keybinds
        .iter()
        .find(|k| k.key == "T" && k.modifiers.contains(&Modifier::Mod));

    assert!(terminal_bind.is_some(), "Should find Mod+T keybind");

    let keybind = terminal_bind.unwrap();
    assert!(keybind.description.is_some());
    assert!(keybind.description.as_ref().unwrap().contains("Terminal"));
}

#[test]
fn test_niri_special_keys() {
    let config_path = PathBuf::from("tests/niri-default-config.kdl");
    let source = NiriSource::new(config_path);

    let keybinds = source.discover().expect("Failed to parse config");

    let volume_up = keybinds.iter().find(|k| k.key == "XF86AudioRaiseVolume");

    assert!(
        volume_up.is_some(),
        "Should find XF86AudioRaiseVolume keybind"
    );

    let keybind = volume_up.unwrap();
    assert_eq!(keybind.modifiers, vec![]);
    assert_eq!(keybind.program, "niri");
}

#[test]
fn test_niri_multiple_modifiers() {
    let config_path = PathBuf::from("tests/niri-default-config.kdl");
    let source = NiriSource::new(config_path);

    let keybinds = source.discover().expect("Failed to parse config");

    let multi_mod = keybinds.iter().find(|k| {
        k.modifiers.contains(&Modifier::Mod)
            && k.modifiers.contains(&Modifier::Shift)
            && k.modifiers.contains(&Modifier::Ctrl)
    });

    assert!(multi_mod.is_some(), "Should find keybind with 3+ modifiers");
}
