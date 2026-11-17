use axum::{
    body::Body,
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::TryStreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;

#[derive(Deserialize)]
pub struct ProxyQuery {
    pub url: String,
    pub cookie: Option<String>,
    pub referer: Option<String>,
}

pub async fn proxy_media_handler(
    headers: HeaderMap,
    Query(query): Query<ProxyQuery>,
    client: Arc<Client>,
) -> Response {
    if query.url.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing url parameter").into_response();
    }

    let mut req_builder = client.get(&query.url);

    req_builder = req_builder
        .header(header::USER_AGENT, "ExoPlayerLib/2.19.1")
        .header(header::ACCEPT, "*/*");

    if let Some(cookie) = &query.cookie {
        let cookie_str = if cookie.contains("signCookie=") {
            cookie.to_string()
        } else {
            format!("signCookie={}", cookie)
        };
        req_builder = req_builder.header(header::COOKIE, cookie_str);
    }

    if let Some(referer) = &query.referer {
        req_builder = req_builder.header(header::REFERER, referer);
    }

    // Forward incoming Range header from browser
    if let Some(range_val) = headers.get(header::RANGE) {
        req_builder = req_builder.header(header::RANGE, range_val);
    }

    let upstream_res = match req_builder.send().await {
        Ok(res) => res,
        Err(e) => {
            error!("Proxy fetch error for {}: {}", query.url, e);
            return (StatusCode::BAD_GATEWAY, format!("Upstream error: {}", e)).into_response();
        }
    };

    let status = upstream_res.status();
    let upstream_headers = upstream_res.headers().clone();

    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        "GET, HEAD, OPTIONS".parse().unwrap(),
    );
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "Range, Content-Type, Accept, Origin".parse().unwrap(),
    );
    response_headers.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        "Content-Range, Content-Length, Accept-Ranges".parse().unwrap(),
    );
    response_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());

    if let Some(ct) = upstream_headers.get(header::CONTENT_TYPE) {
        response_headers.insert(header::CONTENT_TYPE, ct.clone());
    } else {
        response_headers.insert(header::CONTENT_TYPE, "video/mp4".parse().unwrap());
    }

    if let Some(cr) = upstream_headers.get(header::CONTENT_RANGE) {
        response_headers.insert(header::CONTENT_RANGE, cr.clone());
    }

    if let Some(cl) = upstream_headers.get(header::CONTENT_LENGTH) {
        response_headers.insert(header::CONTENT_LENGTH, cl.clone());
    }

    let stream = upstream_res.bytes_stream().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    });
    let body = Body::from_stream(stream);

    let mut resp = Response::new(body);
    *resp.status_mut() = status;
    *resp.headers_mut() = response_headers;

    resp
}
