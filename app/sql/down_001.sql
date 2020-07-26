-----
-- DBバージョン:1 のロールバックファイル

-----
-- 価格データ格納テーブルを削除する
DROP TABLE ohlcv;

-- バージョン情報を削除する
DELETE FROM version WHERE version = 1;
