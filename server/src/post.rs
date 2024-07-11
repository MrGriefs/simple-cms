pub struct Post<'a> {
  id: u32,
  created_at: u32,
  updated_at: u32,
  md5: &'a str,
  source_path: &'a str,
  file_ext: &'a str,
  tags: &'a str,
  description: &'a str,
}