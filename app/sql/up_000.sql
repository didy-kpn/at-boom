-----
-- DBバージョン:0 のマイグレーションファイル

-----
-- 現在のDBバージョンを格納するテーブル
CREATE TABLE IF NOT EXISTS version(
  version     INTEGER   NOT NULL,
  registered  TIMESTAMP NOT NULL DEFAULT (strftime('%s', 'now')),
  UNIQUE(version)
);

-- 現在のバージョンを挿入する
INSERT INTO version(version) VALUES(0);
