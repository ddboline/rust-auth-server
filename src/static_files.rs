use actix_web::{http::StatusCode, HttpRequest, HttpResponse};

use crate::app::AppState;

pub fn index_html(_: HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

pub fn main_css(_: HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/css; charset=utf-8")
        .body(include_str!("../static/main.css"))
}

pub fn register_html(_: HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/register.html"))
}

pub fn main_js(_: HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/javascript; charset=utf-8")
        .body(include_str!("../static/main.js"))
}

pub fn login_html(_: HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/login.html"))
}
