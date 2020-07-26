extern crate atb_db;
extern crate clap;

// ohlcvテーブルの取得条件構造体
struct OhlcvSetting {
    exchange: String,
    pair: String,
    period: String,
    after: i64,
}

fn main() {
    // 対象データベースに接続する
    let result_atbdb = atb_db::AtbDB::connect(None);
    if let Err(err) = result_atbdb {
        eprintln!("{}", err);
        std::process::exit(1);
    }
    let atbdb = result_atbdb.unwrap();

    // コマンドライン引数を取得する
    let args_matches = get_args_matches();

    // DBから取り出す価格データの設定を取得する
    let ohlcv_setting = get_ohlcv_setting(&atbdb, args_matches);

    // 価格データを取得する
    let fetch_result = fetch_ohlcv_from_cryptowat(&ohlcv_setting);

    // 取得に失敗した場合はエラーメッセージを表示して終了する
    if let Err(err) = fetch_result {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    // 価格データをデータベースに保存する
    let store_result = store_ohlcv_to_database(&atbdb, &ohlcv_setting, fetch_result.unwrap());

    // 保存に失敗した場合はエラーメッセージを表示して終了する
    if let Err(err) = store_result {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    // 正常終了
    std::process::exit(0);
}

// コマンドライン引数を取得する
fn get_args_matches() -> clap::ArgMatches<'static> {
    clap::App::new("fetch-ohlcv-rs")
        .version("0.0.1")
        .author("Didy KUPANHY")
        .about("cryptowatから取得したOHLCVデータをデータベースに保存する")
        .arg(
            clap::Arg::with_name("exchange")
                .help("対象取引所")
                .takes_value(true)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("pair")
                .help("対象通貨")
                .takes_value(true)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("period")
                .help("足の期間(秒指定)")
                .takes_value(true)
                .required(true),
        )
        .get_matches()
}

// DBから取り出す価格データの設定を取得する
fn get_ohlcv_setting(
    atbdb: &atb_db::AtbDB,
    args_matches: clap::ArgMatches<'static>,
) -> OhlcvSetting {
    let exchange = args_matches.value_of("exchange").unwrap().to_string();
    let pair = args_matches.value_of("pair").unwrap().to_string();
    let period = args_matches.value_of("period").unwrap().to_string();

    // データベースから設定条件の最終unixtimeを取得する
    let after = match atbdb.get_last_unixtime_from_ohlcv(&exchange, &pair, &period) {
        Ok(after) => after,
        Err(err) => {
            eprintln!("{}", err);
            1514764800
        }
    };

    OhlcvSetting {
        exchange: exchange,
        pair: pair,
        period: period,
        after: after,
    }
}

// 価格データを取得する
fn fetch_ohlcv_from_cryptowat(
    ohlcv_setting: &OhlcvSetting,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let exchange = &ohlcv_setting.exchange;
    let pair = &ohlcv_setting.pair;
    let period = &ohlcv_setting.period;
    let after = &ohlcv_setting.after;

    let url = format!(
        "https://api.cryptowat.ch/markets/{}/{}/ohlc?periods={}&after={}",
        exchange, pair, period, after
    );
    println!("URL : {}", url);
    let resp = reqwest::blocking::get(&url)?.json::<serde_json::Value>()?;
    Ok(resp)
}

// 価格データをデータベースに保存する
fn store_ohlcv_to_database(
    atbdb: &atb_db::AtbDB,
    ohlcv_setting: &OhlcvSetting,
    resp: serde_json::Value,
) -> atb_db::SqliteResult {
    use chrono::{TimeZone, Utc};

    let exchange = &ohlcv_setting.exchange;
    let pair = &ohlcv_setting.pair;
    let period = &ohlcv_setting.period;
    let after = ohlcv_setting.after;

    // 結果がなければ終了
    if let None = resp.get("result") {
        println!("取得可能なデータがありませんでした。");
        return Ok(());
    }
    if let None = resp["result"].get(period.to_string()) {
        println!("取得可能なデータがありませんでした。");
        return Ok(());
    }
    if resp["result"][period.to_string()].is_array() == false {
        println!("取得可能なデータがありませんでした。");
        return Ok(());
    }

    let len_ohlcv = resp["result"][period.to_string()].as_array().unwrap().len();
    if len_ohlcv == 0 {
        println!("{}件のローソク足データを取得。", len_ohlcv);
        return Ok(());
    }

    // UNIX時刻をYYYY-MM-DD hh:mm:ss 形式に変換する
    let head_data: chrono::DateTime<Utc> = Utc.timestamp(
        resp["result"][period.to_string()][0][0].as_i64().unwrap(),
        0,
    );
    let tail_data: chrono::DateTime<Utc> = Utc.timestamp(
        resp["result"][period.to_string()][len_ohlcv - 1][0]
            .as_i64()
            .unwrap(),
        0,
    );

    // 取得したデータの先頭と末尾の日付を出力する
    println!(
        "先頭データ : {}, {}",
        resp["result"][period.to_string()][0][0].as_i64().unwrap(),
        head_data.to_string()
    );
    println!(
        "末尾データ : {}, {}",
        resp["result"][period.to_string()][len_ohlcv - 1][0]
            .as_i64()
            .unwrap(),
        tail_data.to_string()
    );

    // 最終時刻のレコードは再取得するため削除する
    if after != 1514764800 {
        let _ = atbdb.delete_ohlcv(&exchange, &pair, &period, after)?;
    }

    let mut records = vec![];
    let mut lacks = vec![];

    for i in 0..len_ohlcv {
        let record = (
            resp["result"][period.to_string()][i][1].as_f64().unwrap(),
            resp["result"][period.to_string()][i][2].as_f64().unwrap(),
            resp["result"][period.to_string()][i][3].as_f64().unwrap(),
            resp["result"][period.to_string()][i][4].as_f64().unwrap(),
            resp["result"][period.to_string()][i][5].as_f64().unwrap(),
            resp["result"][period.to_string()][i][0].as_i64().unwrap(),
        );

        records.push(record);
        lacks.push(record.5);
    }

    // 取得したデータをデータベースに保存する
    atbdb.insert_ohlcv_list(&exchange, &pair, &period, &records);

    lacks.sort();

    // 欠損データ調査用の変数
    let i_period = period.parse::<i64>().unwrap();
    let mut head_unixtime = lacks[0] - i_period;
    let mut lack = 0;

    // 欠損件数をカウントする
    for i in 0..lacks.len() {
        // 欠損データがあれば出力する
        head_unixtime += i_period;
        if head_unixtime == lacks[i] {
            continue;
        }
        let h: chrono::DateTime<Utc> = Utc.timestamp(head_unixtime, 0);
        let t: chrono::DateTime<Utc> = Utc.timestamp(lacks[i], 0);
        let now_lack = (lacks[i] - head_unixtime) / i_period;
        println!(
            "{}({})から{}({})までの{}件のデータがありません",
            h, head_unixtime, t, lacks[i], now_lack
        );
        head_unixtime = lacks[i];
        lack += now_lack;
    }

    // 保存したデータ数を出力する
    println!(
        "{}件のローソク足データを取得。欠損データは{}件です",
        len_ohlcv, lack
    );

    Ok(())
}
