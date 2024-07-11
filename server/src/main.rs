use std::{io, process::exit, time::Instant};
use clap::Parser;
use context::Context;

mod cli;
mod context;
mod post;
pub mod util;

fn main() {
  let ctx = {
    match cli::Cli::parse().command {
      cli::Command::Start { data_dir, src_dir, init } => {
        Context::new(&data_dir, &src_dir, init)
      },
      _ => {
        eprintln!("The server must be started first");
        exit(0);
      },
    }
  };

  let stdin = io::stdin();
  loop {
    let mut buf = String::new();
    if let Err(e) = stdin.read_line(&mut buf) {
      eprintln!("{e}");
      continue;
    }
    if None == buf.pop() {
      continue;
    }
    if buf.chars().last() == Some('\r') {
      buf.pop();
    }

    let command = match cli::Cli::try_parse_from(util::parse_lp_cmd_line(&buf)) {
      Err(e) => {
        e.print().unwrap();
        continue;
      },
      Ok(v) => v.command,
    };

    match command {
      cli::Command::Start { data_dir: _, src_dir: _, init: _ } => {
        eprintln!("The server has already started");
      },

      cli::Command::Scan => {
        let now = Instant::now();
        ctx.scan_sources();
        println!("Scanned source directory at {:?} in {:.2?}", &ctx.src_dir, now.elapsed());
      },

      cli::Command::Search { tags } => {
        let mut stmt = ctx.db.con.prepare_cached("\
          SELECT `id`, `source_path`, `tags` FROM `post` WHERE `id` IN (\
            SELECT ROWID FROM `post_fts_idx` WHERE `tags` MATCH ?\
          ) LIMIT 50").unwrap();
        let mut rows = stmt.query_map(
          (tags,),
          |row| Ok((
            row.get::<_, u32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
          ))
        ).unwrap().peekable();
  
        if rows.peek().is_none() {
          println!("nothing here but us chickens");
          continue;
        }
  
        for row in rows {
          let (id, source_path, tags) = row.unwrap();
          println!("{id} | {source_path} | {tags}");
        }
      },

      cli::Command::Update { id, set_tags } => {
        let mut stmt = ctx.db.con.prepare_cached("UPDATE `post` SET `tags` = ? WHERE `id` = ?").unwrap();
        let c = stmt.execute((set_tags, id)).unwrap();
        println!("{id} | updated {c}");
      },
    }
  }
}
