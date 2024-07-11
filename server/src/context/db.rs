use std::{env, fs, path::PathBuf, time::{Duration, SystemTime}};
use rusqlite::Connection;

use super::Context;

const SCHEMA_VERSION: u32 = 0;

#[derive(Debug)]
pub enum DbError {
  InitExists,
  UninitNotExists,
  SchemaVersionTooHigh,
  SchemaVersionTooLow,
}

pub struct Database {
  pub con: Connection,
  pub epoch: u32,
}
impl Database {
  pub fn new(path: &PathBuf, init: bool) -> Result<Database, DbError> {
    let path = Self::path(path);

    if !init && !path.is_file() {
      return Err(DbError::UninitNotExists);
    }

    if init && path.is_file() {
      return Err(DbError::InitExists);
    }

    let con = Connection::open(path).unwrap();

    if init {
      Self::init(&con);
    }

    let (v, epoch): (u32, u32) = con.query_row(
      "SELECT schema_version, epoch FROM meta", (),
      |row| {
        Ok((row.get_unwrap(0), row.get_unwrap(1)))
      }).unwrap();

    Self::migrate_schema(&con, v)?;

    Ok(Database {
      con,
      epoch,
    })
  }

  pub fn path(path: &PathBuf) -> PathBuf {
    path.join("vault.db")
  }

  fn init(con: &Connection) {
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    dir.push("data");
    dir.push("schema.sql");

    con.execute_batch(fs::read_to_string(dir).unwrap().as_str()).unwrap();
  }

  fn migrate_schema(_con: &Connection, v: u32) -> Result<(), DbError> {
    if v == SCHEMA_VERSION { return Ok(()) }
    if v > SCHEMA_VERSION {
      return Err(DbError::SchemaVersionTooHigh);
    }

    Err(DbError::SchemaVersionTooLow)
  }

  pub fn does_source_exist(&self, source_path: &str) -> bool {
    let mut stmt = self.con.prepare_cached("\
      SELECT COUNT(1) FROM `post_source` WHERE `source_path` = ? LIMIT 1\
    ").unwrap();
    let count = stmt.query_row((source_path,), |row| row.get::<_, u32>(0)).unwrap();

    return count > 0;
  }

  pub fn add_file(&self, ctx: &Context, path: &PathBuf, md5: &String) {
    #[cfg(debug_assertions)] unsafe {
      static mut COUNTER: u32 = 0;
      COUNTER += 1;
      println!("{} {md5:?} {path:?}", COUNTER);
    }

    let file_size: u32 = path.metadata().unwrap().len().try_into().unwrap();
    let source_path = path.strip_prefix(&ctx.src_dir).unwrap().to_str().unwrap();

    let file_ext;
    match path.extension() {
      None => { return },
      Some(ext) => file_ext = ext.to_str().unwrap().to_lowercase(),
    }

    {
      let created_at: u32 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH + Duration::new(self.epoch as u64, 0))
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap();

      let mut stmt = self.con.prepare_cached("\
        INSERT INTO `post` (`created_at`, `updated_at`, `md5`, `source_path`, `file_ext`, `file_size`, `tags`) \
        VALUES (?, ?, ?, ?, ?, ?, ?) \
        ON CONFLICT (`md5`) DO \
        UPDATE SET \
          `updated_at` = ?1,\
          `tags` = ('has_duplicates ' || `tags`),\
          `source_path` = ?4 \
        WHERE \
          `id` NOT IN (\
            SELECT ROWID FROM `post_fts_idx` WHERE \
              `tags` MATCH 'has_duplicates'\
          ) AND \
          `source_path` != ?4\
      ").unwrap();
      if 0 == stmt.execute((created_at, created_at, md5, source_path, file_ext, file_size, "missing_tags")).unwrap() {
        self.con.prepare_cached("\
          INSERT OR IGNORE INTO `post_source` (`md5`, `source_path`) VALUES (?, ?)\
        ")
        .unwrap()
        .execute((md5, source_path))
        .unwrap();
      }
    }
  }
}