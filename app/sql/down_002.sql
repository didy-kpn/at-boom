-----
-- DBバージョン:2 のロールバックファイル

-----
-- bot管理テーブルを削除する
DROP TABLE bot;

-- バージョン情報を削除する
DELETE FROM version WHERE version = 2;

