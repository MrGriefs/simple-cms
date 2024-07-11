use std::{fs, path::PathBuf};
use md5::{Digest, Md5};

pub fn pathbuf_to_md5(path: &PathBuf) -> String {  
  let mut md5 = Md5::new();
  let mut file = fs::File::open(path).unwrap();

  std::io::copy(&mut file, &mut md5).unwrap();
  let md5 = md5.finalize();
  base16ct::lower::encode_string(&md5)
}