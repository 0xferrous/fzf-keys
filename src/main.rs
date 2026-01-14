use clap::Parser;
use fzf_keys::source::Source;
use fzf_keys::sources::kitty::KittySource;
use fzf_keys::sources::niri::NiriSource;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fzf-keys")]
#[command(about = "Search through keybinds from various programs", long_about = None)]
struct Args {
    /// Path to niri config file
    #[arg(short, long)]
    niri_config: Option<PathBuf>,

    /// Include kitty keybinds (requires kitty terminal)
    #[arg(short, long)]
    kitty: bool,
}

fn main() {
    let args = Args::parse();

    // Collect keybinds from all requested sources
    let mut all_keybinds = Vec::new();

    // Try niri if specified or as default
    if !args.kitty {
        let niri_source = if let Some(config_path) = args.niri_config {
            NiriSource::new(config_path)
        } else {
            match NiriSource::from_default_config() {
                Ok(source) => source,
                Err(e) => {
                    eprintln!("Error initializing niri source: {}", e);
                    return;
                }
            }
        };

        match niri_source.discover() {
            Ok(keybinds) => all_keybinds.extend(keybinds),
            Err(e) => eprintln!("Error discovering niri keybinds: {}", e),
        }
    }

    // Try kitty if specified
    if args.kitty {
        let kitty_source = KittySource::new();
        match kitty_source.discover() {
            Ok(keybinds) => all_keybinds.extend(keybinds),
            Err(e) => eprintln!("Error discovering kitty keybinds: {}", e),
        }
    }

    // Output all keybinds
    for keybind in all_keybinds {
        println!("{}", keybind);
    }
}
