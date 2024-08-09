use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None, after_help = r#"Examples:
"#)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct Cli {
    /// Config file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
    /// Bot token
    #[arg(short, long, global = true)]
    pub token: Option<String>,
    /// Operations
    #[command(subcommand)]
    pub command: Option<SubCommand>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SubCommand {
    /// delete all Mars record from a chat
    #[clap(alias("d"))]
    Delete { chat_id: String },
    /// Export default config.
    #[clap(alias("e"))]
    Export,
}
