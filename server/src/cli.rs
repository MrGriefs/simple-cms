use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueHint};

/// Simple, portable and open source content management system.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand, Debug)]
#[command(about, long_about = None)]
pub enum Command {
  /// Modify existing posts
  #[command()]
  Update {
    #[arg(long)]
    id: u32,

    #[arg(long)]
    set_tags: String,
  },

  /// Check and add new files from source directory
  #[command()]
  Scan,

  /// Search all posts
  #[command()]
  Search {
    #[arg(long)]
    tags: String,
  },

  /// Start the server
  #[command()]
  Start {
    #[arg(short, long, value_hint = ValueHint::DirPath)]
    data_dir: PathBuf,

    #[arg(short, long, value_hint = ValueHint::DirPath)]
    src_dir: PathBuf,

    #[arg(long, default_value_t = false)]
    init: bool,
  },
}
