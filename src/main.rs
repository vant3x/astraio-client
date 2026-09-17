mod ai;
mod cli;
mod cookie;
mod data;
mod error;
mod export;
mod http_client;
mod import;
mod openapi;
mod persistence;
mod protocols;
mod services;
mod ui;
mod utils;

use clap::Parser;

fn main() {
    env_logger::init();

    if std::env::args().len() > 1 {
        let cli = cli::Cli::parse();
        if let Err(e) = cli::run(cli) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else {
        let _ = ui::app::main();
    }
}
