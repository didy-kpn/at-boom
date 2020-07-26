extern crate atb_db;
extern crate clap;

// コマンドの実行モード
enum Command {
    Add,
    Update,
    Remove,
    Get,
    List,
    NoCommand,
}

// コマンドの実行モードとオプションを格納する構造体
struct Config {
    command: Command,
    option: std::collections::HashMap<String, String>,
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

    // 実行コマンドとオプションを取得する
    let config = get_config(args_matches);

    // コマンドを実行する
    let result = actual_main(&atbdb, config);

    // 終了する
    std::process::exit(result);
}

fn _clap_name() -> clap::Arg<'static, 'static> {
    clap::Arg::with_name("name")
        .help("bot名")
        .long("name")
        .takes_value(true)
}

fn _clap_description() -> clap::Arg<'static, 'static> {
    clap::Arg::with_name("description")
        .help("botの説明")
        .long("description")
        .takes_value(true)
}

fn _clap_enable() -> clap::Arg<'static, 'static> {
    clap::Arg::with_name("enable")
        .help("有効かどうか")
        .long("enable")
        .possible_values(&["true", "false"])
        .takes_value(true)
}

fn _clap_long_order() -> clap::Arg<'static, 'static> {
    clap::Arg::with_name("long_order")
        .help("ロング注文可能かどうか")
        .long("long_order")
        .possible_values(&["true", "false"])
        .takes_value(true)
}

fn _clap_short_order() -> clap::Arg<'static, 'static> {
    clap::Arg::with_name("short_order")
        .help("ショート注文可能かどうか")
        .long("short_order")
        .possible_values(&["true", "false"])
        .takes_value(true)
}

fn _clap_operation() -> clap::Arg<'static, 'static> {
    clap::Arg::with_name("operation")
        .help("運用")
        .long("operation")
        .possible_values(&["backtest", "forwardtest", "product"])
        .takes_value(true)
}

fn _clap_output() -> clap::ArgGroup<'static> {
    clap::ArgGroup::with_name("output").args(&["json", "yaml"])
}

// コマンドライン引数を取得する
fn get_args_matches() -> clap::ArgMatches<'static> {
    clap::App::new("bot-admin")
        .version("0.0.1")
        .author("Didy KUPANHY")
        .about("bot管理コマンド")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .setting(clap::AppSettings::DeriveDisplayOrder)
        .subcommand(
            clap::SubCommand::with_name("add")
                .about("管理するbotを追加する")
                .setting(clap::AppSettings::DeriveDisplayOrder)
                .arg(_clap_name().required(true))
                .arg(_clap_description().required(true))
                .arg(_clap_enable())
                .arg(_clap_long_order())
                .arg(_clap_short_order())
                .arg(_clap_operation()),
        )
        .subcommand(
            clap::SubCommand::with_name("update")
                .about("対象botの情報を更新する")
                .setting(clap::AppSettings::DeriveDisplayOrder)
                .arg(clap::Arg::with_name("id").help("対象bot id").required(true))
                .arg(_clap_name())
                .arg(_clap_description())
                .arg(_clap_enable())
                .arg(_clap_long_order())
                .arg(_clap_short_order())
                .arg(_clap_operation()),
        )
        .subcommand(
            clap::SubCommand::with_name("remove")
                .about("管理から対象botを取り除く")
                .arg(clap::Arg::with_name("id").help("対象bot id").required(true)),
        )
        .subcommand(
            clap::SubCommand::with_name("get")
                .about("対象botを取得する")
                .args_from_usage(
                    "-j, --json 'json mode: output group'
                                  -y, --yaml 'yaml mode: output group'",
                )
                .arg(clap::Arg::with_name("id").help("対象bot id").required(true))
                .group(_clap_output()),
        )
        .subcommand(
            clap::SubCommand::with_name("list")
                .about("管理してるbot一覧")
                .args_from_usage(
                    "-j, --json 'json mode: output group'
                                  -y, --yaml 'yaml mode: output group'",
                )
                .group(_clap_output().required(true)),
        )
        .get_matches()
}

fn _get_option(
    args_matches: &clap::ArgMatches<'static>,
    must_keys: &Vec<&str>,
    optional_keys: &Vec<&str>,
) -> std::collections::HashMap<String, String> {
    let mut option = std::collections::HashMap::new();

    // 必須オプションを取得
    for key in must_keys {
        option.insert(
            String::from(*key),
            args_matches.value_of(key).unwrap().to_string(),
        );
    }

    // 任意オプションを取得
    for key in optional_keys {
        if let Some(opt) = args_matches.value_of(key) {
            option.insert(String::from(*key), opt.to_string());
        }
    }

    // json/yamlオプションを取得
    for key in ["json", "yaml"].iter() {
        if args_matches.is_present(key) {
            option.insert(String::from(*key), "1".to_string());
        }
    }

    option
}

// 実行コマンドを取得する
fn get_config(args_matches: clap::ArgMatches<'static>) -> Config {
    // Addコマンドのオプション取得
    if let Some(ref args_matches) = args_matches.subcommand_matches("add") {
        // サブコマンドのオプションのリスト
        let must_keys = vec!["name", "description"];
        let optional_keys = vec!["enable", "long_order", "short_order", "operation"];

        // サブコマンドのオプションを取得する
        let option = _get_option(&args_matches, &must_keys, &optional_keys);

        return Config {
            command: Command::Add,
            option: option,
        };
    }

    // Updateコマンドのオプション取得
    if let Some(ref args_matches) = args_matches.subcommand_matches("update") {
        // サブコマンドのオプションのリスト
        let must_keys = vec!["id"];
        let optional_keys = vec![
            "id",
            "name",
            "description",
            "enable",
            "long_order",
            "short_order",
            "operation",
        ];

        // サブコマンドのオプションを取得する
        let option = _get_option(&args_matches, &must_keys, &optional_keys);

        return Config {
            command: Command::Update,
            option: option,
        };
    }

    // Removeコマンドのオプション取得
    if let Some(ref args_matches) = args_matches.subcommand_matches("remove") {
        // サブコマンドのオプションのリスト
        let must_keys = vec!["id"];
        let optional_keys = vec![];

        // サブコマンドのオプションを取得する
        let option = _get_option(&args_matches, &must_keys, &optional_keys);

        return Config {
            command: Command::Remove,
            option: option,
        };
    }

    // Getコマンドのオプション取得
    if let Some(ref args_matches) = args_matches.subcommand_matches("get") {
        // サブコマンドのオプションのリスト
        let must_keys = vec!["id"];
        let optional_keys = vec![];

        // サブコマンドのオプションを取得する
        let option = _get_option(&args_matches, &must_keys, &optional_keys);

        return Config {
            command: Command::Get,
            option: option,
        };
    }

    // Listコマンドのオプション取得
    if let Some(ref args_matches) = args_matches.subcommand_matches("list") {
        // サブコマンドのオプションのリスト
        let must_keys = vec![];
        let optional_keys = vec![];

        // サブコマンドのオプションを取得する
        let option = _get_option(&args_matches, &must_keys, &optional_keys);

        return Config {
            command: Command::List,
            option: option,
        };
    }

    let option = std::collections::HashMap::new();
    Config {
        command: Command::NoCommand,
        option: option,
    }
}

// コマンドを実行する
fn actual_main(atbdb: &atb_db::AtbDB, config: Config) -> i32 {
    let result = match config.command {
        Command::Add => _add(atbdb, &config.option),
        Command::Update => _update(atbdb, &config.option),
        Command::Remove => _remove(atbdb, config.option),
        Command::Get => _get(atbdb, config.option),
        Command::List => _list(atbdb, config.option),
        _ => Err(atb_db::SqliteError::ExecuteReturnedResults),
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        1
    } else {
        0
    }
}

// Addコマンドを実行する
fn _add(
    atbdb: &atb_db::AtbDB,
    option: &std::collections::HashMap<String, String>,
) -> Result<usize, atb_db::SqliteError> {
    let mut option = option.clone();

    // enableが指定されていれば、enable値を1/0に変換する
    if let Some(v) = option.get("enable") {
        let value = if v == "true" { "1" } else { "0" }.to_string();
        option.insert("enable".to_string(), value);
    }

    // long_orderが指定されていれば、値を1/0に変換する
    if let Some(v) = option.get("long_order") {
        let value = if v == "true" { "1" } else { "0" }.to_string();
        option.insert("long_order".to_string(), value);
    }

    // short_orderが指定されていれば、値を1/0に変換する
    if let Some(v) = option.get("short_order") {
        let value = if v == "true" { "1" } else { "0" }.to_string();
        option.insert("short_order".to_string(), value);
    }

    // 新しいbotデータを追加する
    atbdb.insert_bot(&option)
}

// Updateコマンドを実行する
fn _update(
    atbdb: &atb_db::AtbDB,
    option: &std::collections::HashMap<String, String>,
) -> Result<usize, atb_db::SqliteError> {
    let mut option = option.clone();

    // idを取得する
    let id_value = option.remove("id");

    // enableが指定されていれば、値を1/0に変換する
    if let Some(v) = option.get("enable") {
        let value = if v == "true" { "1" } else { "0" }.to_string();
        option.insert("enable".to_string(), value);
    }

    // long_orderが指定されていれば、値を1/0に変換する
    if let Some(v) = option.get("long_order") {
        let value = if v == "true" { "1" } else { "0" }.to_string();
        option.insert("long_order".to_string(), value);
    }

    // short_orderが指定されていれば、値を1/0に変換する
    if let Some(v) = option.get("short_order") {
        let value = if v == "true" { "1" } else { "0" }.to_string();
        option.insert("short_order".to_string(), value);
    }

    // 対象botデータを更新する
    atbdb.update_bot(&id_value.unwrap(), &option)
}

// Removeコマンドを実行する
fn _remove(
    atbdb: &atb_db::AtbDB,
    option: std::collections::HashMap<String, String>,
) -> Result<usize, atb_db::SqliteError> {
    // idを取得する
    let id_value = option.get("id");

    // 対象botデータを削除する
    atbdb.delete_bot(&id_value.unwrap())
}

// Getコマンドを実行する
fn _get(
    atbdb: &atb_db::AtbDB,
    option: std::collections::HashMap<String, String>,
) -> Result<usize, atb_db::SqliteError> {
    // idを取得する
    let id_value = option.get("id");

    // 対象botデータを削除する
    let result = atbdb.get_bot(&id_value.unwrap());
    if let Err(err) = result {
        return Err(err);
    }

    let bot = result.unwrap();
    let bot_id = bot.get_id() as usize;

    // jsonが指定されていればjson形式で返す
    if option.get("json").is_some() {
        println!("{}", serde_json::to_string(&bot).unwrap());
        return Ok(bot_id);
    }

    // yamlが指定されていればyaml形式で返す
    if option.get("yaml").is_some() {
        println!("{}", serde_yaml::to_string(&bot).unwrap());
        return Ok(bot_id);
    }

    Ok(bot_id)
}

// Listコマンドを実行する
fn _list(
    atbdb: &atb_db::AtbDB,
    option: std::collections::HashMap<String, String>,
) -> Result<usize, atb_db::SqliteError> {
    // botデータの一覧を取得する
    let result = atbdb.get_bot_list();
    if let Err(err) = result {
        return Err(err);
    }

    let bot_list = result.unwrap();
    let bot_list_len = bot_list.get_list_len();

    // jsonが指定されていればjson形式で返す
    if option.get("json").is_some() {
        println!("{}", serde_json::to_string(&bot_list).unwrap());
        return Ok(bot_list_len);
    }

    // yamlが指定されていればyaml形式で返す
    if option.get("yaml").is_some() {
        println!("{}", serde_yaml::to_string(&bot_list).unwrap());
        return Ok(bot_list_len);
    }

    Ok(bot_list_len)
}
