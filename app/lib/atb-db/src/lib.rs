extern crate r2d2;
extern crate r2d2_sqlite;
extern crate read_atb_config;
extern crate rusqlite;

pub type SqliteResult = rusqlite::Result<()>;
pub type SqliteError = rusqlite::Error;

#[allow(dead_code)]
pub struct AtbDB {
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Bot {
    id: i64,
    name: String,
    description: String,
    enable: bool,
    registered: i64,
    token: String,
    long_order: bool,
    short_order: bool,
    operate_type: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BotForGet {
    name: String,
    description: String,
    enable: bool,
    registered: i64,
    long_order: bool,
    short_order: bool,
    operate_type: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BotList {
    bot: Vec<Bot>,
}

#[derive(serde::Serialize)]
pub struct Ohlcv {
    ohlcv: Vec<(f64, f64, f64, f64, f64, i64)>,
}

impl AtbDB {
    #[allow(dead_code)]
    pub fn connect(option_atbconf: Option<read_atb_config::AtbConf>) -> Result<AtbDB, String> {
        let atbconf = if option_atbconf.is_none() {
            // 環境変数を読み込む
            let atbconf = read_atb_config::AtbConf::load_conf();
            if atbconf.is_none() {
                return Err("環境変数`PATH_ATB_CONFIG`を確認ください".to_string());
            }
            atbconf
        } else {
            option_atbconf
        };

        // 環境変数から対象データベースのpathを取得する
        let path_db_file = atbconf.unwrap().get_sqlite3_file();
        if let None = path_db_file {
            return Err(
                "環境変数`PATH_ATB_CONFIG`に指定したファイルを確認してください".to_string(),
            );
        }

        // 対象データベースへのコネクションを取得する
        let manager = r2d2_sqlite::SqliteConnectionManager::file(path_db_file.unwrap());
        let result_pool = r2d2::Pool::new(manager);
        if let Err(err) = result_pool {
            return Err(err.to_string());
        }

        Ok(AtbDB {
            pool: result_pool.unwrap(),
        })
    }

    // ohlcvテーブルから設定条件の最終unixtimeを取得する
    pub fn get_last_unixtime_from_ohlcv(
        &self,
        exchange: &String,
        pair: &String,
        period: &String,
    ) -> rusqlite::Result<i64> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();

        // 最後のunixtime時刻を取得する
        conn.query_row(
            "select max(unixtime) from ohlcv where exchange = ?1 and pair = ?2 and period = ?3",
            rusqlite::params![&exchange, &pair, &period],
            |row| row.get(0),
        )
    }

    // 条件に該当するレコードを一つだけ削除する
    pub fn delete_ohlcv(
        &self,
        exchange: &String,
        pair: &String,
        period: &String,
        unixtime: i64,
    ) -> rusqlite::Result<usize> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();

        conn.execute(
            "delete from ohlcv where exchange = ?1 and pair = ?2 and period = ?3 and unixtime = ?4 limit 1",
            rusqlite::params![&exchange, &pair, &period, unixtime],
        )
    }

    // 複数のohlcvデータを追加する
    pub fn insert_ohlcv_list(
        &self,
        exchange: &String,
        pair: &String,
        period: &String,
        records: &Vec<(f64, f64, f64, f64, f64, i64)>,
    ) {
        let pool = self.pool.clone();
        let mut conn = pool.get().unwrap();

        // SQLを作成する
        let sql_key = "exchange, pair, period, open, high, low, close, volume, unixtime";
        let sql_value = "?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9";
        let sql_insert = &format!("INSERT INTO ohlcv ({}) VALUES ({})", sql_key, sql_value);

        let tx = conn.transaction().unwrap();
        for record in records {
            let _ = tx.execute(
                sql_insert,
                rusqlite::params![
                    exchange, pair, period, record.0, record.1, record.2, record.3, record.4,
                    record.5
                ],
            );
        }
        let _ = tx.commit();
    }

    // 複数のohlcvデータを取得する
    pub fn get_ohlcv_list(
        &self,
        exchange: &String,
        pair: &String,
        period: &String,
    ) -> Result<Ohlcv, SqliteError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();

        let result_stmt = conn.prepare("SELECT open, high, low, close, volume, unixtime FROM ohlcv WHERE exchange = ?1 and pair = ?2 and period = ?3");
        if result_stmt.is_err() {
            return Err(rusqlite::Error::InvalidQuery);
        }
        let mut stmt = result_stmt.unwrap();

        let result_rows = stmt.query_map(rusqlite::params![exchange, pair, period], |row| {
            Ok((
                row.get::<_, f64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        });
        if result_rows.is_err() {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        let rows = result_rows.unwrap();

        let mut ohlcv = Vec::new();
        for row in rows {
            if row.is_ok() {
                ohlcv.push(row.unwrap())
            }
        }
        Ok(Ohlcv { ohlcv: ohlcv })
    }

    // botデータを追加する
    pub fn insert_bot(
        &self,
        option: &std::collections::HashMap<String, String>,
    ) -> rusqlite::Result<usize> {
        let pool = self.pool.clone();
        let mut conn = pool.get().unwrap();

        // 取得したデータをデータベースに保存するためのSQL
        let sql_key = option.keys().map(|s| &**s).collect::<Vec<_>>().join(",");
        let sql_value = option.keys().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql_insert = &format!("INSERT INTO bot ({}) VALUES ({})", sql_key, sql_value);

        // SQLを実行する
        let tx = conn.transaction().unwrap();
        let result = tx.execute(
            sql_insert,
            option.values().map(|s| s.to_string()).collect::<Vec<_>>(),
        );
        let _ = tx.commit();
        result
    }

    // botデータを更新する
    pub fn update_bot(
        &self,
        id: &String,
        option: &std::collections::HashMap<String, String>,
    ) -> rusqlite::Result<usize> {
        let pool = self.pool.clone();
        let mut conn = pool.get().unwrap();

        // 更新用データをデータベースに上書きするためのSQL
        let sql_set = option
            .keys()
            .enumerate()
            .map(|key| format!("{} = ?{}", key.1, key.0 + 2))
            .collect::<Vec<_>>()
            .join(", ");
        let mut sql_value: Vec<String> = Vec::new();
        sql_value.push(id.to_string());
        for value in option.values() {
            sql_value.push(value.to_string());
        }
        let sql_insert = &format!("UPDATE bot SET {} where id = ?1 limit 1", sql_set);

        // SQLを実行する
        let tx = conn.transaction().unwrap();
        let result = tx.execute(sql_insert, sql_value);
        let _ = tx.commit();
        result
    }

    // botデータを削除する
    pub fn delete_bot(&self, id: &String) -> rusqlite::Result<usize> {
        let pool = self.pool.clone();
        let mut conn = pool.get().unwrap();

        // 指定したIDのbotを削除するSQLを実行する
        let tx = conn.transaction().unwrap();
        let result = tx.execute(
            "delete from bot where id = ?1 limit 1",
            rusqlite::params![&id.to_string()],
        );
        let _ = tx.commit();
        result
    }

    // botデータを取得する
    pub fn get_bot(&self, id: &String) -> Result<Bot, SqliteError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();

        // 指定したIDのbotを取得する
        conn.query_row(
            "select id, name, description, enable, registered, token, long_order, short_order, operate_type from bot where id = ?1 limit 1",
            rusqlite::params![&id.to_string()],
            |row| Ok(Bot {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                description: row.get(2).unwrap(),
                enable: row.get(3).unwrap(),
                registered: row.get(4).unwrap(),
                token: row.get(5).unwrap(),
                long_order: row.get(6).unwrap(),
                short_order: row.get(7).unwrap(),
                operate_type: row.get(8).unwrap(),
            })
        )
    }

    // (API用)botデータを取得する
    pub fn get_bot_for_api(&self, id: &String, token: &String) -> Result<BotForGet, SqliteError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();

        conn.query_row(
            "select name, description, enable, registered, long_order, short_order, operate_type from bot where id = ?1 and token = ?2 limit 1",
            rusqlite::params![&id.to_string(), &token.to_string()],
            |row| Ok(BotForGet {
                name: row.get(0).unwrap(),
                description: row.get(1).unwrap(),
                enable: row.get(2).unwrap(),
                registered: row.get(3).unwrap(),
                long_order: row.get(4).unwrap(),
                short_order: row.get(5).unwrap(),
                operate_type: row.get(6).unwrap(),
            })
        )
    }

    // 複数のbotデータを取得する
    pub fn get_bot_list(&self) -> Result<BotList, SqliteError> {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();

        // bot情報一覧を取得する
        let mut stmt = conn.prepare("SELECT * FROM bot").unwrap();
        let result = stmt.query_map(rusqlite::params![], |row| {
            Ok(Bot {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                description: row.get(2).unwrap(),
                enable: row.get(3).unwrap(),
                registered: row.get(4).unwrap(),
                token: row.get(5).unwrap(),
                long_order: row.get(6).unwrap(),
                short_order: row.get(7).unwrap(),
                operate_type: row.get(8).unwrap(),
            })
        });
        if let Err(err) = result {
            return Err(err);
        }

        Ok(BotList {
            bot: result
                .unwrap()
                .map(|bot| bot.unwrap())
                .collect::<Vec<Bot>>(),
        })
    }
}

impl Bot {
    #[allow(dead_code)]
    pub fn get_id(&self) -> i64 {
        self.id
    }
}

impl BotList {
    #[allow(dead_code)]
    pub fn get_list_len(&self) -> usize {
        self.bot.len()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use super::*;
        if let Ok(atbdb) = AtbDB::connect(None) {
            let exchange = "bitflyer".to_string();
            let pair = "btcjpy".to_string();
            let period = "60".to_string();
            println!(
                "{:?}",
                atbdb.get_last_unixtime_from_ohlcv(&exchange, &pair, &period)
            );
        }
        assert_eq!(2 + 2, 3);
    }
}
