use std::fmt::Display;
use std::str::FromStr;

use actix_web::{get, web, App, HttpResponse, HttpServer};
use log::{warn, LevelFilter};
use reqwest::Client;
use serde::Serialize;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use steam_gid::{ExpectLog, GroupData, GroupId64, GroupId8, GroupIdentifier, GroupUrl};

type State = web::Data<Client>;

// JavaScript don't like those 64-bit unsigned integers,
// gotta encode them as strings.
#[derive(Serialize)]
struct WebGroupData {
    name: String,
    id_8: String,
    id_64: String,
    url: String,
}
impl From<GroupData> for WebGroupData {
    fn from(value: GroupData) -> Self {
        WebGroupData {
            name: value.name,
            id_8: value.id_8.0.to_string(),
            id_64: value.id_64.0.to_string(),
            url: value.url.0,
        }
    }
}

fn internal_server_error<E: Display>(msg: &str, err: E) -> HttpResponse {
    warn!("{msg}: {err}");
    HttpResponse::InternalServerError().finish()
}

#[get("/gid-8/{gid_8}")]
async fn get_gid_8(state: State, gid_8: web::Path<u64>) -> HttpResponse {
    let gid_8 = match GroupId8::try_from(gid_8.into_inner()) {
        Err(e) => return internal_server_error("gid-8", e),
        Ok(v) => v,
    };
    let group_data = match GroupData::fetch(&state, GroupIdentifier::Id8(gid_8)).await {
        Err(e) => return internal_server_error("gid-8", e),
        Ok(v) => WebGroupData::from(v),
    };
    HttpResponse::Ok().json(&group_data)
}

#[get("/gid-64/{gid_64}")]
async fn get_gid_64(state: State, gid_64: web::Path<u64>) -> HttpResponse {
    let gid_64 = match GroupId64::try_from(gid_64.into_inner()) {
        Err(e) => return internal_server_error("gid-64", e),
        Ok(v) => v,
    };
    let group_data = match GroupData::fetch(&state, GroupIdentifier::Id64(gid_64)).await {
        Err(e) => return internal_server_error("gid-64", e),
        Ok(v) => WebGroupData::from(v),
    };
    HttpResponse::Ok().json(&group_data)
}

#[get("/url/{urls}")]
async fn get_url(state: State, url: web::Path<String>) -> HttpResponse {
    let url = match GroupUrl::from_str(url.as_str()) {
        Err(e) => return internal_server_error("url", e),
        Ok(v) => v,
    };
    let group_data = match GroupData::fetch(&state, GroupIdentifier::Url(url)).await {
        Err(e) => return internal_server_error("url", e),
        Ok(v) => WebGroupData::from(v),
    };
    HttpResponse::Ok().json(&group_data)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    HttpServer::new(|| {
        App::new().app_data(web::Data::new(Client::new())).service(
            web::scope("/api")
                .service(get_gid_64)
                .service(get_gid_8)
                .service(get_url),
        )
    })
    .bind(("0.0.0.0", 80))
    .expect_log("couldn't bind to socket")
    .run()
    .await
    .expect_log("server tripped");
}
