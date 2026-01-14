use clap::Parser;
use fzf_keys::source::Source;
use fzf_keys::sources::niri::NiriSource;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fzf-keys")]
#[command(about = "Search through keybinds from various programs", long_about = None)]
struct Args {
    /// Path to niri config file
    #[arg(short, long)]
    niri_config: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let source = if let Some(config_path) = args.niri_config {
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

    match source.discover() {
        Ok(keybinds) => {
            for keybind in keybinds {
                println!("{}", keybind);
            }
        }
        Err(e) => eprintln!("Error discovering keybinds: {}", e),
    }
}
