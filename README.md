# fzf-keys

Search through keybinds from various programs using fzf.

## Usage

```bash
cargo run --quiet | fzf
```

## How Sources Work

Sources implement the `Source` trait to discover keybinds from different programs:

```rust
pub trait Source {
    fn name(&self) -> &str;
    fn discover(&self) -> Result<Vec<Keybind>, Box<dyn std::error::Error>>;
}
```

Each source:
1. Reads config files for a specific program
2. Parses keybind definitions
3. Returns a list of `Keybind` structs

## Current Sources

### Niri (`sources/niri.rs`)

- **Config location**: `~/.config/niri/config.kdl`
- **Format**: KDL (parsed with v1-fallback for compatibility)
- **Parsing**: Finds `binds { }` blocks and extracts keybind nodes
- **Keybind format**: `Mod+Shift+Key { action; }`
- **Supports**: Modifier combinations, special keys (XF86*), descriptions from `hotkey-overlay-title`

## Adding New Sources

1. Create a new file in `src/sources/`
2. Implement the `Source` trait
3. Parse your program's config format
4. Return `Keybind` structs with modifiers, key, action, description, and program name

Example:
```rust
impl Source for MyProgramSource {
    fn name(&self) -> &str { "myprogram" }

    fn discover(&self) -> Result<Vec<Keybind>, Box<dyn std::error::Error>> {
        // Read config, parse keybinds, return Vec<Keybind>
    }
}
```
