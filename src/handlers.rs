use crate::mapping::format_tab_sections;
use crate::models::{ApiResponse, SectionList};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PageQuery {
    pub page: Option<i32>,
    pub q: Option<String>,
    pub tab_id: Option<i32>,
    #[serde(rename = "tabId")]
    pub tab_id_camel: Option<i32>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub quality: Option<String>,
    pub resource_id: Option<String>,
}

fn get_session_id(headers: &HeaderMap) -> Option<String> {
    if let Some(cookie_hdr) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        for part in cookie_hdr.split(';') {
            let part = part.trim();
            if part.starts_with("session_id=") {
                return Some(part["session_id=".len()..].to_string());
            }
        }
    }
    None
}

pub async fn root_handler() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "moviebox-api-rs",
        "engine": "Rust / Axum + Tokio",
        "version": "1.0.0"
    }))
}

pub async fn health_handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn home_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let page = query.page.unwrap_or(1);

    let raw = state.client.get_tab_content(1, page, Some(&sess.token)).await;
    let items = raw
        .get("data")
        .and_then(|d| d.get("list").or_else(|| d.get("items")).or_else(|| d.get("subjects")))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let sections = format_tab_sections(&items);

    let mut resp = Json(ApiResponse {
        code: 0,
        message: None,
        msg: None,
        data: SectionList { list: sections },
    })
    .into_response();

    // Set cookie if needed
    if get_session_id(&headers).is_none() {
        if let Ok(cval) = format!("session_id={}; Path=/; HttpOnly; SameSite=Lax", sess.id).parse() {
            resp.headers_mut().insert(header::SET_COOKIE, cval);
        }
    }

    resp
}

pub async fn movies_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Json<ApiResponse<SectionList>> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let page = query.page.unwrap_or(1);

    let raw = state.client.get_tab_content(2, page, Some(&sess.token)).await;
    let items = raw
        .get("data")
        .and_then(|d| d.get("list").or_else(|| d.get("items")).or_else(|| d.get("subjects")))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let sections = format_tab_sections(&items);

    Json(ApiResponse {
        code: 0,
        message: None,
        msg: None,
        data: SectionList { list: sections },
    })
}

pub async fn anime_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Json<ApiResponse<SectionList>> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let page = query.page.unwrap_or(1);

    let raw = state.client.get_tab_content(8, page, Some(&sess.token)).await;
    let items = raw
        .get("data")
        .and_then(|d| d.get("list").or_else(|| d.get("items")).or_else(|| d.get("subjects")))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let sections = format_tab_sections(&items);

    Json(ApiResponse {
        code: 0,
        message: None,
        msg: None,
        data: SectionList { list: sections },
    })
}

pub async fn tab_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tab_id): Path<i32>,
    Query(query): Query<PageQuery>,
) -> Json<ApiResponse<SectionList>> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let page = query.page.unwrap_or(1);

    let raw = state.client.get_tab_content(tab_id, page, Some(&sess.token)).await;
    let items = raw
        .get("data")
        .and_then(|d| d.get("list").or_else(|| d.get("items")).or_else(|| d.get("subjects")))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let sections = format_tab_sections(&items);

    Json(ApiResponse {
        code: 0,
        message: None,
        msg: None,
        data: SectionList { list: sections },
    })
}

pub async fn generic_category_handler(
    cat_id: i32,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Json<ApiResponse<SectionList>> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let page = query.page.unwrap_or(1);

    let raw = state.client.get_tab_content(cat_id, page, Some(&sess.token)).await;
    let items = raw
        .get("data")
        .and_then(|d| d.get("list").or_else(|| d.get("items")).or_else(|| d.get("subjects")))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let sections = format_tab_sections(&items);

    Json(ApiResponse {
        code: 0,
        message: None,
        msg: None,
        data: SectionList { list: sections },
    })
}

pub async fn short_tv_handler(state: State<AppState>, headers: HeaderMap, query: Query<PageQuery>) -> Json<ApiResponse<SectionList>> {
    generic_category_handler(3, state, headers, query).await
}

pub async fn kids_handler(state: State<AppState>, headers: HeaderMap, query: Query<PageQuery>) -> Json<ApiResponse<SectionList>> {
    generic_category_handler(4, state, headers, query).await
}

pub async fn education_handler(state: State<AppState>, headers: HeaderMap, query: Query<PageQuery>) -> Json<ApiResponse<SectionList>> {
    generic_category_handler(5, state, headers, query).await
}

pub async fn music_handler(state: State<AppState>, headers: HeaderMap, query: Query<PageQuery>) -> Json<ApiResponse<SectionList>> {
    generic_category_handler(6, state, headers, query).await
}

pub async fn asian_handler(state: State<AppState>, headers: HeaderMap, query: Query<PageQuery>) -> Json<ApiResponse<SectionList>> {
    generic_category_handler(7, state, headers, query).await
}

pub async fn western_handler(state: State<AppState>, headers: HeaderMap, query: Query<PageQuery>) -> Json<ApiResponse<SectionList>> {
    generic_category_handler(9, state, headers, query).await
}

pub async fn nollywood_handler(state: State<AppState>, headers: HeaderMap, query: Query<PageQuery>) -> Json<ApiResponse<SectionList>> {
    generic_category_handler(10, state, headers, query).await
}

pub async fn game_handler(state: State<AppState>, headers: HeaderMap, query: Query<PageQuery>) -> Json<ApiResponse<SectionList>> {
    generic_category_handler(11, state, headers, query).await
}

pub async fn search_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Json<Value> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let q = query.q.unwrap_or_default();
    let page = query.page.unwrap_or(1);

    let raw = state.client.search(&q, page, Some(&sess.token)).await;
    let items = raw
        .get("data")
        .and_then(|d| d.get("list").or_else(|| d.get("items")).or_else(|| d.get("subjects")))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mapped: Vec<crate::models::MovieItem> = items.iter().map(crate::mapping::map_item).collect();

    Json(json!({
        "code": 0,
        "data": {
            "list": mapped
        }
    }))
}

pub async fn search_suggestions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<Value> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let raw = state.client.get_search_suggestions(Some(&sess.token)).await;

    let items = raw
        .get("data")
        .and_then(|d| d.get("list").or_else(|| d.get("items")))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let suggestions: Vec<String> = items
        .iter()
        .filter_map(|it| {
            it.get("keyword")
                .or_else(|| it.get("title"))
                .or_else(|| it.get("name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    Json(json!({
        "code": 0,
        "data": suggestions
    }))
}

pub async fn detail_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(subject_id): Path<String>,
) -> Json<Value> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let raw = state.client.get_detail(&subject_id, Some(&sess.token)).await;
    Json(raw)
}

pub async fn episodes_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(series_id): Path<String>,
) -> Json<Value> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let raw = state.client.get_episodes(&series_id, Some(&sess.token)).await;
    Json(raw)
}

pub async fn rankings_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Json<Value> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let tab_id = query.tab_id_camel.or(query.tab_id).unwrap_or(1);

    let raw = state.client.get_rankings(tab_id, Some(&sess.token)).await;
    let mut sections = Vec::new();

    if let Some(data) = raw.get("data") {
        if let Some(subjs) = data.get("subjects").and_then(|v| v.as_array()) {
            let items: Vec<crate::models::MovieItem> = subjs.iter().take(10).map(crate::mapping::map_item).collect();
            sections.push(json!({
                "title": "Top Rankings",
                "items": items
            }));
        } else if let Some(lists) = data.get("lists").or_else(|| data.get("list")).and_then(|v| v.as_array()) {
            for l in lists {
                let title = l.get("name").or_else(|| l.get("title")).and_then(|v| v.as_str()).unwrap_or("Rankings");
                if let Some(arr) = l.get("items").or_else(|| l.get("list")).and_then(|v| v.as_array()) {
                    let items: Vec<crate::models::MovieItem> = arr.iter().take(10).map(crate::mapping::map_item).collect();
                    sections.push(json!({
                        "title": title,
                        "items": items
                    }));
                }
            }
        }
    }

    Json(json!({
        "code": 0,
        "data": sections
    }))
}

pub async fn stream_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(subject_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let season = query.season.unwrap_or(1);
    let episode = query.episode.unwrap_or(1);

    let raw = state.client.get_play_info(&subject_id, season, episode, Some(&sess.token)).await;
    let data = raw.get("data").cloned().unwrap_or(json!({}));

    let streams = data
        .get("streams")
        .or_else(|| data.get("streamList"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if streams.is_empty() {
        return Ok(Json(json!({
            "code": 404,
            "msg": "No streams found",
            "data": {}
        })));
    }

    // Pick first working or best stream
    let best_stream = streams[0].clone();
    let raw_url = best_stream.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let cookie = best_stream
        .get("signCookie")
        .or_else(|| data.get("signCookie"))
        .or_else(|| raw.get("signCookie"))
        .and_then(|v| v.as_str())
        .unwrap_or(&sess.token)
        .to_string();

    let host_hdr = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:8000");

    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");

    let base_url = format!("{}://{}", proto, host_hdr);
    let encoded_url = urlencoding::encode(&raw_url);
    let encoded_cookie = urlencoding::encode(&cookie);

    let proxied_url = format!(
        "{}/proxy-media?url={}&cookie={}",
        base_url, encoded_url, encoded_cookie
    );

    let subtitles = data
        .get("subTitleList")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let is_hls = raw_url.to_lowercase().contains(".m3u8");
    let is_dash = raw_url.to_lowercase().contains(".mpd");

    Ok(Json(json!({
        "url": proxied_url,
        "raw_url": raw_url,
        "cookie": cookie,
        "duration": 3600,
        "subtitles": subtitles,
        "isHls": is_hls,
        "isDash": is_dash,
        "headers": {
            "User-Agent": "ExoPlayerLib/2.19.1",
            "Cookie": cookie
        }
    })))
}

pub async fn user_info_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<Value> {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    if let Some(u) = sess.user_info {
        Json(json!({
            "logged_in": true,
            "user": u
        }))
    } else {
        Json(json!({
            "logged_in": false,
            "user": null
        }))
    }
}

pub async fn login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<crate::models::LoginPayload>,
) -> Response {
    let sess = state.get_session(get_session_id(&headers).as_deref()).await;
    let res = state.client.login(&payload.account, &payload.password, payload.auth_type).await;

    if res.get("status").and_then(|v| v.as_str()) == Some("success") {
        if let Some(user) = res.get("user") {
            let token = user.get("token").and_then(|v| v.as_str()).unwrap_or(&sess.token).to_string();
            let uid = user.get("userId").or_else(|| user.get("uid")).and_then(|v| {
                if let Some(s) = v.as_str() { Some(s.to_string()) }
                else if let Some(n) = v.as_i64() { Some(n.to_string()) }
                else { None }
            }).unwrap_or_else(|| sess.user_id.clone());

            // Permanently save account to disk
            crate::state::save_account_to_disk(&token, &uid, Some(user));

            let mut write_guard = state.sessions.write().await;
            if let Some(s) = write_guard.get_mut(&sess.id) {
                s.token = token.clone();
                s.user_id = uid.clone();
                s.user_info = Some(user.clone());
            }

            // Also update global guest token so every subsequent session uses this account
            let mut global_guard = state.guest_token.write().await;
            *global_guard = (token, uid);
        }
    }

    let mut resp = Json(res).into_response();
    if let Ok(cval) = format!("session_id={}; Path=/; HttpOnly; SameSite=None; Secure; Max-Age=31536000", sess.id).parse() {
        resp.headers_mut().insert(header::SET_COOKIE, cval);
    }
    resp
}

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<Value> {
    if let Some(sid) = get_session_id(&headers) {
        let mut write_guard = state.sessions.write().await;
        if let Some(s) = write_guard.get_mut(&sid) {
            let guard = state.guest_token.read().await;
            s.token = guard.0.clone();
            s.user_id = guard.1.clone();
            s.user_info = None;
        }
    }
    Json(json!({"status": "success"}))
}

pub async fn history_handler() -> Json<Value> {
    Json(json!({
        "code": 0,
        "data": {
            "list": []
        }
    }))
}

pub async fn watchlist_handler() -> Json<Value> {
    Json(json!({
        "code": 0,
        "data": {
            "list": []
        }
    }))
}
