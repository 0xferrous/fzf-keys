use fzf_keys::source::Source;
use fzf_keys::sources::niri::NiriSource;

fn main() {
    match NiriSource::from_default_config() {
        Ok(source) => match source.discover() {
            Ok(keybinds) => {
                for keybind in keybinds {
                    println!("{}", keybind);
                }
            }
            Err(e) => eprintln!("Error discovering keybinds: {}", e),
        },
        Err(e) => eprintln!("Error initializing niri source: {}", e),
    }
}
