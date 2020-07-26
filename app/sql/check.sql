CREATE TABLE version(
  version     INTEGER   NOT NULL,
  registered  TIMESTAMP NOT NULL DEFAULT (strftime('%s', 'now')),
  UNIQUE(version)
);
CREATE TABLE ohlcv(
  exchange  TEXT      NOT NULL,  -- 取引所
  pair      TEXT      NOT NULL,  -- 取引通貨
  period    INTEGER   NOT NULL,  -- 足の期間
  open      INTEGER   NOT NULL,  -- 始値
  high      INTEGER   NOT NULL,  -- 高値
  low       INTEGER   NOT NULL,  -- 安値
  close     INTEGER   NOT NULL,  -- 終値
  volume    INTEGER   NOT NULL,  -- 出来高
  unixtime  TIMESTAMP NOT NULL   -- UNIX時間
);
CREATE INDEX idx_exchange ON ohlcv(exchange, pair, period);
CREATE TABLE bot(
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
