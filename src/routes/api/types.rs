use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    message: Option<String>,
    data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(message: &str, data: T) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    /// Success with a message but no payload.
    pub fn ok_msg(message: &str) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
            data: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
            data: None,
        }
    }
}

#[derive(Deserialize)]
pub struct DownloadQuery {
    pub token: Option<String>,
    /// When true, serve with `Content-Disposition: inline` so the browser
    /// renders the file (image, PDF) in-page instead of downloading it.
    pub inline: Option<bool>,
}

#[derive(Deserialize)]
pub struct ShareRequestBody {
    pub expires_in_minutes: Option<i64>,
}

#[derive(Serialize)]
pub struct ShareResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct FileSearchQuery {
    pub filename: String,
}

#[derive(Deserialize)]
pub struct RenameRequestBody {
    pub name: String,
}

#[derive(Deserialize)]
pub struct VisibilityRequestBody {
    pub public: bool,
}
