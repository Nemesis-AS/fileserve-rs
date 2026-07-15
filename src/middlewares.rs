use actix_web::{
    Error, HttpResponse,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

use crate::routes::api::types::ApiResponse;

const TUS_VERSION: &str = "1.0.0";

pub async fn tus_resumable(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let resumable = req
        .headers()
        .get("Tus-Resumable")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_owned());

    let Some(resumable) = resumable else {
        let response = HttpResponse::PreconditionFailed()
            .append_header(("Tus-Resumable", TUS_VERSION))
            .json(ApiResponse::error("Missing Tus-Resumable header"));
        return Ok(req.into_response(response).map_into_boxed_body());
    };

    if resumable != TUS_VERSION {
        let response = HttpResponse::PreconditionFailed()
            .append_header(("Tus-Resumable", TUS_VERSION))
            .json(ApiResponse::error("Unsupported TUS version"));
        return Ok(req.into_response(response).map_into_boxed_body());
    }

    let mut res = next.call(req).await?;

    res.headers_mut().insert(
        actix_web::http::header::HeaderName::from_static("tus-resumable"),
        actix_web::http::header::HeaderValue::from_static(TUS_VERSION),
    );

    Ok(res.map_into_boxed_body())
}
