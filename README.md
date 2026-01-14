# fzf-keys

Search through keybinds from various programs using fzf.

![Demo](https://github.com/user-attachments/assets/3f50256e-17f6-47dd-bc56-cb29b04690c4)

## Usage

```bash
# Use default config location (~/.config/niri/config.kdl)
cargo run --quiet | fzf

# Or specify a custom config path
cargo run --quiet -- --niri-config /path/to/config.kdl | fzf
```

## How Sources Work

Sources implement the `Source` trait to discover keybinds from different programs:

```rust
pub trait Source {
    type Item: Display;

    fn name(&self) -> &str;
    fn discover(&self) -> Result<Vec<Self::Item>, Box<dyn std::error::Error>>;
}
```

Each source:
1. Defines an associated `Item` type that implements `Display` (e.g., `Keybind`)
2. Reads config files for a specific program
3. Parses keybind definitions
4. Returns a list of items that are formatted for fzf via their `Display` implementation

## Current Sources

### Niri (`sources/niri.rs`)

- **Config location**: `~/.config/niri/config.kdl`
- **Format**: KDL (parsed with v1-fallback for compatibility)
- **Parsing**: Finds `binds { }` blocks and extracts keybind nodes
- **Keybind format**: `Mod+Shift+Key [properties] { action; }`
- **Supported modifiers**: `Mod`, `Super`/`Win`, `Alt`, `Ctrl`/`Control`, `Shift`, `ISO_Level3_Shift`/`Mod5`, `ISO_Level5_Shift`/`Mod3`
- **Supported properties**:
  - `hotkey-overlay-title` - Description shown in overlay
  - `repeat` - Auto-repeat when held (default: true)
  - `cooldown-ms` - Rate limiting in milliseconds
  - `allow-when-locked` - Works when session is locked
  - `allow-inhibiting` - Can be inhibited by applications
- **Special keys**: XF86 keys, mouse buttons, wheel/touchpad scroll events

## Adding New Sources

1. Create a new file in `src/sources/`
2. Define your item type (or reuse `Keybind`) that implements `Display`
3. Implement the `Source` trait with your item type
4. Parse your program's config format
5. Return items that will be formatted via their `Display` implementation

Example:
```rust
impl Source for MyProgramSource {
    type Item = Keybind;

    fn name(&self) -> &str { "myprogram" }

    fn discover(&self) -> Result<Vec<Self::Item>, Box<dyn std::error::Error>> {
        // Read config, parse keybinds, return Vec<Keybind>
    }
}
```

The `Item` type must implement `Display` - this is what gets printed to stdout for fzf to consume.
