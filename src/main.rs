#![allow(unused)]

mod service;
mod system;
mod view;
mod config;

use clap::Parser;
use view::app::{MonitorApp, SearchApp};
use view::engine::TuiEngine;
use signal_hook::consts::signal::SIGWINCH;
use signal_hook::flag;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "Easy Monitor")]
#[command(author = "Hamza Oğuzalp")]
#[command(version = "1.0")]
#[command(about = "A simple system monitoring tool written in Rust", long_about = None)]
struct Cli {
    #[arg(short, long)]
    system: bool,

    #[arg(short, long, value_name = "FILENAME")]
    find: Option<String>,

    #[arg(short, long, default_value_t = String::from("."))]
    directory: String,
}

fn main() {
    let cli = Cli::parse();
    
    let is_resized = Arc::new(AtomicBool::new(false));
    flag::register(SIGWINCH, Arc::clone(&is_resized)).expect("Failed to register signal handler");
    
    if let Some(query) = cli.find {
        let mut app = SearchApp::new(
            PathBuf::from(&cli.directory),
            query,
        );
        let mut engine = TuiEngine::new();
        engine.run(&mut app).expect("Failed to run TUI engine");
    } else if cli.system {
        let mut app = MonitorApp::new();
        let mut engine = TuiEngine::new();
        engine.run(&mut app).expect("Failed to run TUI engine");
    }
}
