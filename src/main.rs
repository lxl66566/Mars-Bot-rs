#![feature(async_closure)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(missing_docs)]
#![allow(clippy::module_name_repetitions)]

mod bot;
mod cli;
mod config;
mod db;
mod utils;

use clap::Parser;
use cli::{Cli, SubCommand};
use die_exit::DieWith;

use crate::db::{DbOperation, DB};

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            SubCommand::Delete { chat_id } => DB
                .drop_table(chat_id.as_str())
                .die_with(|e| format!("drop table {chat_id} failed: {e:?}")),
        }
    } else {
        bot::run(cli).await;
    };
}
