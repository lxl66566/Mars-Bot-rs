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

use std::{
    panic,
    sync::Mutex,
    time::{Duration, Instant},
};

use clap::Parser;
use cli::{Cli, SubCommand};
use die_exit::DieWith;
use log::error;
use utils::DATA_ROOT_PATH;

use crate::db::{DbOperation, DB};

static RETRY_INTERVAL: Duration = Duration::from_secs(30);

static LAST_PANIC: Mutex<Option<Instant>> = Mutex::new(None);

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
    std::fs::create_dir_all(DATA_ROOT_PATH.as_path())
        .die_with(|e| format!("create data dir `{DATA_ROOT_PATH:?}` failed: {e:?}"));
    let cli = Box::leak(Box::new(Cli::parse()));
    panic::set_hook(Box::new(|info| {
        let last_panic;
        {
            let mut binding = LAST_PANIC.lock().unwrap();
            last_panic = *binding.get_or_insert_with(Instant::now);
        }
        if last_panic.elapsed() > RETRY_INTERVAL {
            eprintln!("panic interval > {RETRY_INTERVAL:?}, exiting. info: {info:?}");
            std::process::exit(1);
        } else {
            error!("mars-bot panic: {info:?}, retrying...");
            LAST_PANIC.lock().unwrap().replace(Instant::now());
            retry(cli.clone());
        }
    }));
    retry(cli.clone());
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
