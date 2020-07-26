extern crate read_atb_config;
extern crate atb_db;

use actix_web;

use std::sync::Arc;

// サーバー設定
struct ServerConfig {
    host: String,
    port: String,
}

#[derive(serde::Deserialize)]
struct GetId {
    id: i64,
}

fn main() {
    // 環境変数から設定ファイルを読み込む
    let result_atbconf = read_atb_config::AtbConf::load_conf();
    if let None = result_atbconf {
        eprintln!("環境変数`PATH_ATB_CONFIG`を確認ください");
        std::process::exit(1);
    }

    // 対象データベースに接続する
    let result_atbdb = atb_db::AtbDB::connect(result_atbconf.clone());
    if let Err(err) = result_atbdb {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    let atbconf = result_atbconf.unwrap();
    let atbdb = result_atbdb.unwrap();

    // サーバー設定情報を取得する
    let result_server_config = get_server_config(atbconf);
    if let None = result_server_config {
        eprintln!("環境変数`PATH_ATB_CONFIG`に指定したファイルを確認してください");
        std::process::exit(1);
    }
    let server_config = result_server_config.unwrap();

    // webサーバーを起動する
    let result = actual_main(atbdb, server_config);

    // 終了する
    std::process::exit(result);
}

// サーバー設定情報を取得する
fn get_server_config(atbconf: read_atb_config::AtbConf) -> Option<ServerConfig> {
    if atbconf.has_api_host() == false || atbconf.has_api_port() == false {
        return None;
    }

    Some(ServerConfig {
        host: atbconf.get_api_host().unwrap().to_string(),
        port: atbconf.get_api_port().unwrap().to_string(),
    })
}

// webサーバーを起動する
fn actual_main(atbdb: atb_db::AtbDB, server_config: ServerConfig) -> i32 {
    if let Err(err) = run(Arc::new(atbdb), server_config) {
        eprintln!("{}", err);
        1
    } else {
        0
    }
}

// APIサーバーを起動する
#[actix_rt::main]
async fn run(atbdb: Arc<atb_db::AtbDB>, server_config: ServerConfig) -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info,actix_web=info");
    env_logger::init();

    let addr = format!("{}:{}", server_config.host, server_config.port);

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_web::middleware::Logger::default())
            .data(actix_web::web::JsonConfig::default().limit(4096))
            .data(atbdb.clone())
            .service(actix_web::web::resource("/").route(actix_web::web::get().to(index)))
            .service(
                actix_web::web::resource("/ohlcv/{market}/{pair}/{period}")
                    .route(actix_web::web::get().to(get_ohlcv)),
            )
            .service(actix_web::web::resource("/bot").route(actix_web::web::post().to(post_bot)))
            .service(actix_web::web::resource("/bot/{id}").route(actix_web::web::get().to(get_bot)))
    })
    .bind(addr)?
    .run()
    .await
}

fn _get_index() -> Result<String, String> {
    Ok(r#"
        GET /ohlcv/{market}/{pair}/{period}
        GET /bot/{bot-id}
        POST /bot
    "#
    .to_string())
}

async fn index() -> Result<actix_web::HttpResponse, actix_web::Error> {
    let res = actix_web::web::block(move || _get_index())
        .await
        .map(|body| actix_web::HttpResponse::Ok().body(body))
        .map_err(|_| actix_web::HttpResponse::InternalServerError())?;
    Ok(res)
}

async fn get_ohlcv(
    path: actix_web::web::Path<(String, String, i64)>,
    atbdb: actix_web::web::Data<Arc<atb_db::AtbDB>>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let res = actix_web::web::block(move || {
        atbdb.get_ohlcv_list(&path.0, &path.1, &path.2.to_string())
    })
    .await
    .map(|ohlcv| actix_web::HttpResponse::Ok().json(ohlcv))
    .map_err(|_| actix_web::HttpResponse::InternalServerError())?;
    Ok(res)
}

fn _get_option(
    json: serde_json::Value,
) -> Option<std::collections::HashMap<String, String>> {
    let mut option: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // name チェック
    if json.get("name").is_none()
        || !json["name"].is_string()
        || json["name"].as_str().unwrap().len() == 0
    {
        return None;
    }
    option.insert(
        "name".to_string(),
        json["name"].as_str().unwrap().to_string(),
    );

    // description チェック
    if json.get("description").is_none()
        || !json["description"].is_string()
        || json["description"].as_str().unwrap().len() == 0
    {
        return None;
    }
    option.insert(
        "description".to_string(),
        json["description"].as_str().unwrap().to_string(),
    );

    // bool チェック
    if json.get("enable").is_none() || !json["enable"].is_boolean() {
        return None;
    }
    option.insert(
        "enable".to_string(),
        if json["enable"].as_bool().unwrap() {
            "1"
        } else {
            "0"
        }
        .to_string(),
    );

    // long_orderが指定されていれば、値を1/0に変換する
    if json.get("long_order").is_some() && json["long_order"].is_boolean() {
        let value = if json["long_order"].as_bool().unwrap() {
            "1"
        } else {
            "0"
        }
        .to_string();
        option.insert("long_order".to_string(), value);
    }

    // short_orderが指定されていれば、値を1/0に変換する
    if json.get("short_order").is_some() && json["short_order"].is_boolean() {
        let value = if json["short_order"].as_bool().unwrap() {
            "1"
        } else {
            "0"
        }
        .to_string();
        option.insert("short_order".to_string(), value);
    }

    Some(option)
}

async fn post_bot(
    body: actix_web::web::Bytes,
    atbdb: actix_web::web::Data<Arc<atb_db::AtbDB>>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let result = std::str::from_utf8(&body).unwrap();
    let json: serde_json::Value = serde_json::from_str(result).unwrap();

    let res = actix_web::web::block(move || {
        let result_option = _get_option(json);
        if result_option.is_none() {
            return Err("".to_string());
        }
        let option = result_option.unwrap();

        let result_last_id = atbdb.insert_bot(&option);
        if result_last_id.is_err() {
            return Err("".to_string());
        }
        let last_id = result_last_id.unwrap();

        let result_bot = atbdb.get_bot(&last_id.to_string());
        if result_bot.is_err() {
            return Err("".to_string());
        }

        Ok(result_bot.unwrap())
    })
    .await
    .map(|bot| actix_web::HttpResponse::Ok().json(bot))
    .map_err(|_| actix_web::HttpResponse::InternalServerError())?;
    Ok(res)
}

fn _get_token(req: actix_web::HttpRequest) -> String {
    if req.headers().get("token").is_some() {
        let token_header = req.headers().get("token");
        token_header.unwrap().to_str().unwrap()
    } else {
        ""
    }
    .to_string()
}

async fn get_bot(
    req: actix_web::HttpRequest,
    path: actix_web::web::Path<GetId>,
    atbdb: actix_web::web::Data<Arc<atb_db::AtbDB>>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let token = _get_token(req);
    let res = actix_web::web::block(move || {
        atbdb.get_bot_for_api(&path.id.to_string(), &token)
    })
    .await
    .map(|bot| actix_web::HttpResponse::Ok().json(bot))
    .map_err(|_| actix_web::HttpResponse::InternalServerError())?;
    Ok(res)
}
