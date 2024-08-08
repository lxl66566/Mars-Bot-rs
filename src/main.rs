#![feature(async_closure)]
#![feature(try_blocks)]
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
use utils::DATA_ROOT_PATH;

use crate::db::DB;

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
    std::fs::create_dir_all(DATA_ROOT_PATH.as_path())
        .die_with(|e| format!("create data dir `{DATA_ROOT_PATH:?}` failed: {e:?}"));
    let cli = Cli::parse();
    retry(cli);
}

#[tokio::main]
async fn retry(cli: Cli) {
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
