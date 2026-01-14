# fzf-keys

Search through keybinds from various programs using fzf.

![Demo](https://github.com/user-attachments/assets/3f50256e-17f6-47dd-bc56-cb29b04690c4)

## Usage

```bash
# Use default niri config location (~/.config/niri/config.kdl)
cargo run --quiet | fzf

# Or specify a custom niri config path
cargo run --quiet -- --niri-config /path/to/config.kdl | fzf

# Search kitty keybinds (requires Python with kitty installed)
cargo run --quiet -- --kitty | fzf

# Or use nix develop shell (includes all dependencies)
nix develop --command bash -c "cargo run --quiet -- --kitty | fzf"
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

### Kitty (`sources/kitty.rs`)

- **Discovery method**: Uses PyO3 to call kitty's Python API directly
- **Requirements**:
  - Python 3.13+ with kitty installed
  - PyO3 for Rust-Python interop
- **How it works**:
  - Calls `kitty.config.load_config()` to load kitty configuration
  - Directly accesses `opts.keyboard_modes` to get all keybindings
  - Detects the actual `kitty_mod` value (e.g., `ctrl+shift`) using `mod_to_names()`
  - Expands all shortcuts with their real modifiers (shows `Ctrl+Shift+c` instead of `kitty_mod+c`)
  - Converts each keybind to human-readable format using kitty's own `Shortcut.human_repr()` method
- **Advantages**:
  - Gets actual runtime keybinds (not just config file)
  - Shows defaults, custom, and changed shortcuts
  - Detects and expands `kitty_mod` to show the actual key combination
  - No config file parsing needed - uses kitty's own config parser
  - More reliable than parsing text output
- **Supported modifiers**: `ctrl`/`control`, `shift`, `alt`/`opt`/`option`, `super`/`cmd`/`command` (all expanded from `kitty_mod`)
- **Features**:
  - Multi-key sequences: `ctrl+f>2`
  - Actions with arguments captured in full
  - Special keys handled: `+`, function keys, arrow keys, etc.

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
