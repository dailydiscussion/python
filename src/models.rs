use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieItem {
    #[serde(rename = "subjectId")]
    pub subject_id: String,
    pub id: String,
    pub title: String,
    pub cover: String,
    pub poster: String,
    pub score: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(rename = "subjectType")]
    pub subject_type: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<i64>,
    #[serde(rename = "seeTime", skip_serializing_if = "Option::is_none")]
    pub see_time: Option<i64>,
    #[serde(rename = "seenStatus", skip_serializing_if = "Option::is_none")]
    pub seen_status: Option<i64>,
    #[serde(rename = "likeStatus")]
    pub like_status: u32,
    pub description: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
    #[serde(rename = "categoryId", skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    #[serde(rename = "deepLink")]
    pub deep_link: String,
    #[serde(rename = "isPost")]
    pub is_post: bool,
    #[serde(rename = "streamUrl", skip_serializing_if = "Option::is_none")]
    pub stream_url: Option<String>,
    #[serde(rename = "isCollection")]
    pub is_collection: bool,
    pub items: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub items: Vec<MovieItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionList {
    pub list: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginPayload {
    pub account: String,
    pub password: String,
    #[serde(rename = "authType", default = "default_auth_type")]
    pub auth_type: i32,
}

fn default_auth_type() -> i32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RegisterPayload {
    pub account: String,
    pub password: String,
    pub otp: String,
    #[serde(rename = "authType", default = "default_auth_type")]
    pub auth_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct OtpPayload {
    pub account: String,
    #[serde(rename = "authType", default = "default_auth_type")]
    pub auth_type: i32,
    #[serde(rename = "type", default = "default_auth_type")]
    pub otp_type: i32,
}
