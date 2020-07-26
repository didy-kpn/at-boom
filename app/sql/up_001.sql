-----
-- DBバージョン:1 のマイグレーションファイル

-- 現在のバージョンを挿入する
INSERT INTO version(version) VALUES(1);

-----
-- 価格データ格納テーブル。
CREATE TABLE IF NOT EXISTS ohlcv(
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

-- INDEXを設定する
CREATE INDEX IF NOT EXISTS idx_exchange ON ohlcv(exchange, pair, period);
