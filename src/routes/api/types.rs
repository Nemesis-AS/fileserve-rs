use serde::Serialize;

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

    pub fn error_data(message: &str, data: T) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
            data: Some(data),
        }
    }
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
