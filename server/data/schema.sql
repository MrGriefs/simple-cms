CREATE TABLE post (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  md5 VARCHAR NOT NULL,
  source_path VARCHAR NOT NULL,
  file_ext VARCHAR NOT NULL,
  file_size INTEGER NOT NULL,
  tags TEXT NOT NULL,
  description TEXT DEFAULT '' NOT NULL
);

CREATE UNIQUE INDEX post_idx_md5 ON post (md5);
CREATE INDEX post_idx_source_path ON post (source_path);

CREATE TABLE post_source (
  md5 VARCHAR NOT NULL,
  source_path VARCHAR PRIMARY KEY NOT NULL
);

CREATE INDEX post_source_idx_md5 ON post_source (md5);

-- usage:
-- SELECT ROWID FROM post_fts_idx WHERE tags MATCH 'good_quality NOT photography';
CREATE VIRTUAL TABLE post_fts_idx USING fts5(tags, content='post', content_rowid='id');

CREATE TRIGGER post_ai AFTER INSERT ON post BEGIN
  INSERT INTO post_fts_idx(rowid, tags) VALUES (new.id, new.tags);
  INSERT INTO post_source(md5, source_path) VALUES (new.md5, new.source_path);
END;
CREATE TRIGGER post_ad AFTER DELETE ON post BEGIN
  INSERT INTO post_fts_idx(post_fts_idx, rowid, tags) VALUES('delete', old.id, old.tags);
  DELETE FROM post_source WHERE md5 = old.md5;
END;
CREATE TRIGGER post_au_tags AFTER UPDATE OF tags ON post BEGIN
  INSERT INTO post_fts_idx(post_fts_idx, rowid, tags) VALUES('delete', old.id, old.tags);
  INSERT INTO post_fts_idx(rowid, tags) VALUES (new.id, new.tags);
END;
CREATE TRIGGER post_au_source_path AFTER UPDATE OF source_path ON post BEGIN
  INSERT OR IGNORE INTO post_source(md5, source_path) VALUES (new.md5, new.source_path);
END;

CREATE TABLE tag (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  name VARCHAR NOT NULL
);

CREATE TABLE meta (
  schema_version INTEGER NOT NULL,
  epoch INTEGER NOT NULL
);
INSERT INTO meta VALUES (0, strftime('%s'));
