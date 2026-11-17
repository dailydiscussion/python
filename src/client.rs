use crate::utils::{
    current_timestamp_ms, generate_client_token, generate_tr_signature, get_default_client_info,
    md5_hex, DEFAULT_GUEST_TOKEN,
};
use reqwest::{header::HeaderMap, Client, Method};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{error, info, warn};

pub const BASE_URL: &str = "https://apig.inmoviebox.com";

#[derive(Clone)]
pub struct MovieBoxClient {
    http: Client,
    pub base_url: String,
}

impl MovieBoxClient {
    pub fn new() -> Self {
        let mut builder = Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .pool_max_idle_per_host(20)
            .gzip(true);

        // Optional proxy check
        if let Ok(proxy_str) = std::env::var("PROXY_URL") {
            if !proxy_str.is_empty() {
                if let Ok(proxy) = reqwest::Proxy::all(&proxy_str) {
                    builder = builder.proxy(proxy);
                    info!("Rust Client initialized with upstream proxy: {}", proxy_str);
                }
            }
        }

        Self {
            http: builder.build().unwrap_or_default(),
            base_url: BASE_URL.to_string(),
        }
    }

    pub fn build_headers(
        &self,
        timestamp: u64,
        token: Option<&str>,
        _user_id: Option<&str>,
    ) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(
            "User-Agent",
            "MovieBox/4.0.02 (Android 14; Pixel 6)".parse().unwrap(),
        );
        headers.insert("Accept", "application/json".parse().unwrap());
        headers.insert("X-M-Version", "4.0.02".parse().unwrap());
        headers.insert("Referer", format!("{}/", self.base_url).parse().unwrap());
        headers.insert("X-Sign-Version", "2.0".parse().unwrap());
        headers.insert("X-Client-Status", "0".parse().unwrap());
        headers.insert("X-Play-Mode", "2".parse().unwrap());
        headers.insert("appid", "4U01pxRu278GqCZKY9".parse().unwrap());
        headers.insert("region", "IN".parse().unwrap());
        headers.insert("lang", "en".parse().unwrap());
        headers.insert("os", "android".parse().unwrap());
        headers.insert("X-Timestamp", timestamp.to_string().parse().unwrap());

        let client_info = serde_json::to_string(&get_default_client_info()).unwrap_or_default();
        if let Ok(val) = client_info.parse() {
            headers.insert("X-Client-Info", val);
        }

        if let Some(tok) = token {
            if !tok.is_empty() {
                if let Ok(auth_val) = format!("Bearer {}", tok).parse() {
                    headers.insert("Authorization", auth_val);
                    headers.insert("X-Client-Status", "0".parse().unwrap());
                }
            } else {
                headers.insert("X-Client-Token", generate_client_token().parse().unwrap());
                headers.insert("X-Client-Status", "1".parse().unwrap());
            }
        } else {
            headers.insert("X-Client-Token", generate_client_token().parse().unwrap());
            headers.insert("X-Client-Status", "1".parse().unwrap());
        }

        headers
    }

    pub async fn request(
        &self,
        method: Method,
        endpoint: &str,
        params: Option<&HashMap<String, String>>,
        body: Option<&Value>,
        token: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<Value, String> {
        let mut full_params: HashMap<String, String> = params.cloned().unwrap_or_default();
        if !full_params.contains_key("host") {
            full_params.insert("host".to_string(), "apig.inmoviebox.com".to_string());
        }

        let mut sorted_keys: Vec<&String> = full_params.keys().collect();
        sorted_keys.sort();
        let query_string = sorted_keys
            .iter()
            .map(|k| format!("{}={}", k, full_params.get(*k).unwrap()))
            .collect::<Vec<_>>()
            .join("&");

        let body_str = body
            .map(|b| serde_json::to_string(b).unwrap_or_default())
            .unwrap_or_default();

        let timestamp = current_timestamp_ms();
        let raw_url = format!("{}{}", self.base_url, endpoint);
        let sig_url = format!("{}?{}", raw_url, query_string);

        let signature = generate_tr_signature(method.as_str(), &sig_url, &body_str, timestamp);

        let mut headers = self.build_headers(timestamp, token, user_id);
        headers.insert("Content-Type", "application/json;charset=UTF-8".parse().unwrap());
        headers.insert("Host", "apig.inmoviebox.com".parse().unwrap());
        headers.insert("x-tr-signature", signature.parse().unwrap());

        if !body_str.is_empty() {
            headers.insert("Content-Length", body_str.len().to_string().parse().unwrap());
        }

        let mut req = self
            .http
            .request(method.clone(), &raw_url)
            .headers(headers.clone())
            .query(&full_params);

        if method == Method::POST || method == Method::PUT {
            if !body_str.is_empty() {
                req = req.body(body_str.clone());
            }
        }

        let res = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                error!("Request error to {}: {}", endpoint, e);
                return Err(e.to_string());
            }
        };

        // Smart Path Healing: If 404, retry by toggling /wefeed-mobile-bff prefix
        let status = res.status();
        if status.as_u16() == 404 {
            let alt_endpoint = if endpoint.contains("/wefeed-mobile-bff") {
                endpoint.replace("/wefeed-mobile-bff", "")
            } else {
                format!("/wefeed-mobile-bff{}", endpoint)
            };

            info!("Retrying with healed path: {}", alt_endpoint);
            let alt_url = format!("{}{}", self.base_url, alt_endpoint);
            let alt_sig_url = format!("{}?{}", alt_url, query_string);
            let alt_sig = generate_tr_signature(method.as_str(), &alt_sig_url, &body_str, timestamp);

            let mut alt_headers = headers;
            alt_headers.insert("x-tr-signature", alt_sig.parse().unwrap());

            let mut retry_req = self
                .http
                .request(method.clone(), &alt_url)
                .headers(alt_headers)
                .query(&full_params);

            if (method == Method::POST || method == Method::PUT) && !body_str.is_empty() {
                retry_req = retry_req.body(body_str);
            }

            if let Ok(retry_res) = retry_req.send().await {
                if let Ok(val) = retry_res.json::<Value>().await {
                    return Ok(val);
                }
            }
        }

        match res.json::<Value>().await {
            Ok(val) => Ok(val),
            Err(e) => {
                warn!("JSON decode error for {}: {}", endpoint, e);
                Ok(json!({"code": 1, "msg": e.to_string(), "data": {}}))
            }
        }
    }

    pub async fn visitor_login(&self) -> Result<(String, String), String> {
        let res = self
            .request(
                Method::POST,
                "/wefeed-mobile-bff/user-api/visitor-login",
                None,
                Some(&json!({})),
                Some(DEFAULT_GUEST_TOKEN),
                None,
            )
            .await?;

        if res.get("code").and_then(|c| c.as_i64()) == Some(0) {
            if let Some(data) = res.get("data") {
                let token = data
                    .get("token")
                    .and_then(|t| t.as_str())
                    .unwrap_or(DEFAULT_GUEST_TOKEN)
                    .to_string();
                let user_id = data
                    .get("userId")
                    .or_else(|| data.get("uid"))
                    .and_then(|u| {
                        if let Some(s) = u.as_str() {
                            Some(s.to_string())
                        } else if let Some(n) = u.as_i64() {
                            Some(n.to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "7065948410213816232".to_string());
                return Ok((token, user_id));
            }
        }

        Ok((DEFAULT_GUEST_TOKEN.to_string(), "7065948410213816232".to_string()))
    }

    pub async fn get_categories(&self, cat_id: i32, page: i32, token: Option<&str>) -> Value {
        let mut params = HashMap::new();
        params.insert("tabId".to_string(), cat_id.to_string());
        params.insert("page".to_string(), page.to_string());
        params.insert("pageSize".to_string(), "24".to_string());

        self.request(
            Method::GET,
            "/wefeed-mobile-bff/tab-operating",
            Some(&params),
            None,
            token,
            None,
        )
        .await
        .unwrap_or_else(|_| json!({"code": 500, "data": {}}))
    }

    pub async fn get_tab_content(&self, tab_id: i32, page: i32, token: Option<&str>) -> Value {
        self.get_categories(tab_id, page, token).await
    }

    pub async fn search(&self, q: &str, page: i32, token: Option<&str>) -> Value {
        let payload = json!({
            "keyword": q,
            "page": page,
            "perPage": 20,
            "pageSize": 20,
            "subjectType": 0,
            "type": 0,
            "q": q
        });

        // Try V2 search first
        if let Ok(res) = self
            .request(
                Method::POST,
                "/wefeed-mobile-bff/subject-api/search/v2",
                None,
                Some(&payload),
                token,
                None,
            )
            .await
        {
            if res.get("code").and_then(|c| c.as_i64()) == Some(0) {
                return res;
            }
        }

        self.request(
            Method::POST,
            "/wefeed-mobile-bff/subject-api/search",
            None,
            Some(&payload),
            token,
            None,
        )
        .await
        .unwrap_or_else(|_| json!({"code": 500, "data": {}}))
    }

    pub async fn get_detail(&self, subject_id: &str, token: Option<&str>) -> Value {
        let mut params = HashMap::new();
        params.insert("subjectId".to_string(), subject_id.to_string());

        self.request(
            Method::GET,
            "/subject-api/get",
            Some(&params),
            None,
            token,
            None,
        )
        .await
        .unwrap_or_else(|_| json!({"code": 500, "data": {}}))
    }

    pub async fn get_episodes(&self, series_id: &str, token: Option<&str>) -> Value {
        let mut params = HashMap::new();
        params.insert("subjectId".to_string(), series_id.to_string());

        self.request(
            Method::GET,
            "/subject-api/season-info",
            Some(&params),
            None,
            token,
            None,
        )
        .await
        .unwrap_or_else(|_| json!({"code": 500, "data": {}}))
    }

    pub async fn get_play_info(&self, subject_id: &str, season: i32, episode: i32, token: Option<&str>) -> Value {
        let mut params = HashMap::new();
        params.insert("subjectId".to_string(), subject_id.to_string());
        if season > 0 {
            params.insert("se".to_string(), season.to_string());
        }
        if episode > 0 {
            params.insert("ep".to_string(), episode.to_string());
        }

        self.request(
            Method::GET,
            "/wefeed-mobile-bff/subject-api/play-info/v2",
            Some(&params),
            None,
            token,
            None,
        )
        .await
        .unwrap_or_else(|_| json!({"code": 500, "data": {}}))
    }

    pub async fn get_rankings(&self, tab_id: i32, token: Option<&str>) -> Value {
        let mut params = HashMap::new();
        params.insert("tabId".to_string(), tab_id.to_string());
        params.insert("categoryType".to_string(), "all".to_string());
        params.insert("page".to_string(), "1".to_string());
        params.insert("perPage".to_string(), "20".to_string());

        self.request(
            Method::GET,
            "/wefeed-mobile-bff/tab/ranking-list",
            Some(&params),
            None,
            token,
            None,
        )
        .await
        .unwrap_or_else(|_| json!({"code": 500, "data": {}}))
    }

    pub async fn get_search_suggestions(&self, token: Option<&str>) -> Value {
        let mut params = HashMap::new();
        params.insert("page".to_string(), "1".to_string());
        params.insert("pageSize".to_string(), "10".to_string());

        self.request(
            Method::GET,
            "/wefeed-mobile-bff/subject-api/search-rank",
            Some(&params),
            None,
            token,
            None,
        )
        .await
        .unwrap_or_else(|_| json!({"code": 500, "data": {}}))
    }

    pub async fn login(&self, account: &str, password: &str, auth_type: i32) -> Value {
        let hashed_pwd = md5_hex(password.as_bytes());
        let mut payload = json!({
            "password": hashed_pwd,
            "package_name": "com.community.oneroom",
            "authType": auth_type,
            "type": 0
        });

        if account.contains('@') {
            payload["mail"] = json!(account);
            payload["authType"] = json!(1);
        } else {
            payload["phone"] = json!(account);
            payload["authType"] = json!(0);
        }

        self.request(
            Method::POST,
            "/wefeed-mobile-bff/user-api/login",
            None,
            Some(&payload),
            None,
            None,
        )
        .await
        .unwrap_or_else(|_| json!({"code": 500, "msg": "Login failed"}))
    }
}
