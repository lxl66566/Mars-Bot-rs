#![feature(async_closure)]

use clap::Parser;
use cli::Cli;
use log::warn;

mod bot;
mod cli;
mod config;
mod db;
mod utils;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Warn)
        .init();
    let cli = Cli::parse();
}
