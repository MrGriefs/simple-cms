use std::path::PathBuf;
use db::Database;
use rayon::iter::{IntoParallelRefMutIterator, ParallelBridge, ParallelIterator};

mod util;
mod db;

pub struct Context {
  pub db: Database,
  pub data_dir: PathBuf,
  pub src_dir: PathBuf,
}

impl Context {
  pub fn new(data_dir: &PathBuf, src_dir: &PathBuf, init: bool) -> Context {
    Context {
      data_dir: data_dir.clone(),
      src_dir: src_dir.clone(),
      db: Database::new(data_dir, init).unwrap(),
    }
  }

  pub fn scan_sources(&self) {
    self.scan_sources_at(&self.src_dir);
  }

  fn scan_sources_at(&self, path: &PathBuf) {
    struct Item {
      path: PathBuf,
      md5: String,
    }

    let buf: Vec<Item> = jwalk::WalkDir::new(path)
      .skip_hidden(true)
      .follow_links(true)
      .parallelism(jwalk::Parallelism::RayonNewPool(0))
      .into_iter()
      .par_bridge()
      .filter_map(|ent| {
        let ent = ent.unwrap();
        let path = ent.path();
        if !path.is_file() {
          return None;
        }

        Some(Item {
          path,
          md5: String::default(),
        })
      })
      .collect();
    let mut buf: Vec<Item> = buf.into_iter().filter(|item| {
      let source_path = item.path.strip_prefix(&self.src_dir).unwrap().to_str().unwrap();
      !self.db.does_source_exist(source_path)
    }).collect();

    buf.chunks_mut(1024).for_each(|chunk| {
      chunk.par_iter_mut().for_each(|item| item.md5 = util::pathbuf_to_md5(&item.path));
      chunk.into_iter().for_each(|item| self.db.add_file(&self, &item.path, &item.md5));
    });
  }
}
