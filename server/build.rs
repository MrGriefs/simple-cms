use std::{env, path::PathBuf, process::Command};

fn main() {
  let out_dir = env::var_os("OUT_DIR").unwrap();
  let dest_path = {
    let mut p = PathBuf::from(out_dir);
    p.pop();
    p.pop();
    p.pop();
    p
  };

  {
    #[cfg(windows)] {
      Command::new("xcopy")
        .args(&["data", dest_path.join("data").to_str().unwrap(), "/YES"])
    }
    #[cfg(not(windows))] {
      Command::new("cp")
        .args(&["-t", dest_path.to_str().unwrap(), "-r", "data"])
    }
  }.status().unwrap();

  println!("cargo::rerun-if-changed=data");
}