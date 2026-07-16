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

    pub fn ok_msg(data: T) -> Self {
        Self {
            success: false,
            message: None,
            data: Some(data),
        }
    }

    // pub fn error_data(message: &str, data: T) -> Self {
    //     Self {
    //         success: false,
    //         message: Some(message.into()),
    //         data: Some(data),
    //     }
    // }
}

impl ApiResponse<()> {
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
