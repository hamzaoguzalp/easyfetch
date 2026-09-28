mod config;
mod system;
mod view;

use clap::Parser;
use view::app::MonitorApp;
use view::engine::TuiEngine;

#[derive(Parser)]
#[command(name = "easyfetch")]
#[command(author = "Hamza Oğuzalp")]
#[command(version = "1.0")]
#[command(about = "A fast and beautiful system monitor and fetch tool written in Rust", long_about = None)]
struct Cli {
    #[arg(short, long, help = "Run system monitor dashboard")]
    system: bool,

    #[arg(short, long, default_value_t = 1, help = "Refresh interval in seconds")]
    time: u64,

    #[arg(
        long,
        help = "Generate default config file in ~/.config/easyfetch/config.toml"
    )]
    generate_config: bool,

    #[arg(long, help = "Print default config to stdout")]
    print_config: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.print_config {
        print!("{}", config::DEFAULT_CONFIG);
        return;
    }

    if cli.generate_config {
        match config::generate_default_config() {
            Ok(path) => {
                println!(
                    "Successfully created default configuration file at: {}",
                    path.display()
                );
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    let mut app = MonitorApp::new(cli.time);
    let mut engine = TuiEngine::new();
    engine
        .run(&mut app, cli.time)
        .expect("Failed to run TUI engine");
}
