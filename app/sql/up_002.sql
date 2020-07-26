-----
-- DBバージョン:2 のマイグレーションファイル

-- 現在のバージョンを挿入する
INSERT INTO version(version) VALUES(2);

-----
-- アルゴリズム取引botを管理するテーブル
CREATE TABLE IF NOT EXISTS bot(
  -- botID
  id            INTEGER   PRIMARY KEY,

  -- 名前
  name          TEXT      NOT NULL,

  -- 説明文
  description   TEXT      NOT NULL,

  -- 有効かどうか
  enable        BOOLEAN   NOT NULL CHECK(enable in (0, 1)) DEFAULT 1,

  -- 登録日時
  registered    TIMESTAMP NOT NULL DEFAULT (strftime('%s', 'now')),

   -- アクセス用トークン
  token         TEXT      NOT NULL CHECK(length(token) = 36) DEFAULT(printf('%s-%s-%s-%s-%s', lower(hex(randomblob(4))), lower(hex(randomblob(2))), lower(hex(randomblob(2))), lower(hex(randomblob(2))), lower(hex(randomblob(6))))),

  -- ロング注文可能かどうか
  long_order    BOOLEAN   NOT NULL CHECK(long_order in (0, 1)) DEFAULT 1,

  -- ショート注文可能かどうか
  short_order   BOOLEAN   NOT NULL CHECK(short_order in (0, 1)) DEFAULT 0,

  -- 運用段階(バックテスト、フォワードテスト、実運用)
  operate_type  TEXT      NOT NULL CHECK(operate_type in ('backtest', 'forwardtest', 'product')) DEFAULT 'backtest',

  unique(id)
);
