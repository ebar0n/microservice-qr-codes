use actix_files::NamedFile;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use chrono::prelude::Utc;
use chrono::SecondsFormat;
use image::Luma;
use json::JsonValue;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct FormMessage {
    message: String,
}

async fn health(request: HttpRequest) -> Result<HttpResponse, Error> {
    let headers = request.headers();
    let now = Utc::now();

    let response_json: JsonValue = json::object! {
        "health" => "Ok",
        "agent" => format!("{}", match headers.get("user-agent") {
            None => "",
            Some(x) => x.to_str().unwrap(),
        }),
        "created_at" => now.to_rfc3339_opts(SecondsFormat::Millis, false),
        "version" => env!("CARGO_PKG_VERSION"),
    };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(response_json.dump()))
}

async fn api_generate(
    _request: HttpRequest,
    params: web::Query<FormMessage>,
) -> Result<HttpResponse, Error> {
    let token = Uuid::new_v4().to_simple().to_string();
    let name = format!("{}.png", token);
    let filename = format!("/tmp/{}.png", token);
    let fileurl = format!("/static/{}", name);
    let message = &params.message;

    let code = QrCode::new(message.clone()).unwrap();
    let image = code.render::<Luma<u8>>().build();

    image.save(filename.clone()).unwrap();

    let response_json: JsonValue = json::object! {
        "url" => fileurl,
        "message" => message.clone(),
    };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(response_json.dump()))
}

async fn statifiles(request: HttpRequest) -> Result<NamedFile> {
    let path = format!("/tmp/{}", request.match_info().query("filename"));
    // println!("{:?}", path);
    Ok(NamedFile::open(path)?)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/health/").route(web::get().to(health)))
            .service(
                web::scope("/api/v1")
                    .service(web::resource("/generate/").route(web::get().to(api_generate))),
            )
            .service(web::resource("/static/{filename:.*}").route(web::get().to(statifiles)))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header, test, web, App};

    #[actix_rt::test]
    async fn test_api_health_ok() {
        let mut app = test::init_service(App::new().route("/health/", web::get().to(health))).await;
        let req = test::TestRequest::with_header(header::CONTENT_TYPE, "aplication/json")
            .uri("/health/")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_rt::test]
    async fn test_api_generate_fail() {
        let mut app =
            test::init_service(App::new().route("/api/v1/generate/", web::get().to(api_generate)))
                .await;
        let req = test::TestRequest::with_header(header::CONTENT_TYPE, "aplication/json")
            .uri("/api/v1/generate/")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_rt::test]
    async fn test_api_generate_ok() {
        let mut app =
            test::init_service(App::new().route("/api/v1/generate/", web::get().to(api_generate)))
                .await;
        let req = test::TestRequest::with_header(header::CONTENT_TYPE, "aplication/json")
            .uri("/api/v1/generate/?message=hello+wold")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 200);

        let body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };
        let result = json::parse(std::str::from_utf8(body).unwrap());
        let data: JsonValue = match result {
            Ok(x) => x,
            Err(_) => panic!("Response error"),
        };
        let url = data["url"].as_str().unwrap();

        let mut app = test::init_service(
            App::new().route("/static/{filename:.*}", web::get().to(statifiles)),
        )
        .await;
        let req = test::TestRequest::with_header(header::CONTENT_TYPE, "aplication/json")
            .uri(url)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 200);
    }
}
