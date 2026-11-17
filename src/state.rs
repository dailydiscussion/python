use crate::client::MovieBoxClient;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct UserSession {
    pub id: String,
    pub token: String,
    pub user_id: String,
    pub user_info: Option<Value>,
}

#[derive(Clone)]
pub struct AppState {
    pub client: MovieBoxClient,
    pub proxy_client: Arc<Client>,
    pub sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    pub guest_token: Arc<RwLock<(String, String)>>,
}

impl AppState {
    pub async fn new() -> Self {
        let client = MovieBoxClient::new();
        let proxy_client = Arc::new(
            Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .pool_max_idle_per_host(50)
                .build()
                .unwrap_or_default(),
        );

        // Check for configured permanent account token in environment
        let (token, uid) = if let Ok(env_token) = std::env::var("ACCOUNT_TOKEN") {
            if !env_token.trim().is_empty() {
                let env_uid = std::env::var("ACCOUNT_USER_ID").unwrap_or_else(|_| "7065948410213816232".to_string());
                tracing::info!("Initialized backend with permanent ACCOUNT_TOKEN for UID: {}", env_uid);
                (env_token.trim().to_string(), env_uid.trim().to_string())
            } else {
                client.visitor_login().await.unwrap_or_else(|_| {
                    (
                        crate::utils::DEFAULT_GUEST_TOKEN.to_string(),
                        "7065948410213816232".to_string(),
                    )
                })
            }
        } else {
            client.visitor_login().await.unwrap_or_else(|_| {
                (
                    crate::utils::DEFAULT_GUEST_TOKEN.to_string(),
                    "7065948410213816232".to_string(),
                )
            })
        };

        Self {
            client,
            proxy_client,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            guest_token: Arc::new(RwLock::new((token, uid))),
        }
    }

    pub async fn get_session(&self, session_id: Option<&str>) -> UserSession {
        if let Some(sid) = session_id {
            let read_guard = self.sessions.read().await;
            if let Some(sess) = read_guard.get(sid) {
                return sess.clone();
            }
        }

        // Create new guest session
        let sid = uuid::Uuid::new_v4().to_string();
        let (tok, uid) = {
            let guard = self.guest_token.read().await;
            (guard.0.clone(), guard.1.clone())
        };

        let sess = UserSession {
            id: sid.clone(),
            token: tok,
            user_id: uid,
            user_info: None,
        };

        let mut write_guard = self.sessions.write().await;
        write_guard.insert(sid, sess.clone());
        sess
    }
}
