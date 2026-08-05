use std::sync::LazyLock;

use actix_web::{
    Error, HttpResponse,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::{self, HeaderValue},
    middleware::Next,
    web,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

use crate::config::AppConfig;
use crate::routes::api::types::ApiResponse;

const TUS_VERSION: &str = "1.0.0";

/// Content-Security-Policy for the app itself, built once on first use.
///
/// SvelteKit's `index.html` carries a single inline bootstrap script whose text
/// changes with every build, so its hash is derived from the embedded asset at
/// startup rather than hardcoded. That is worth the trouble over a blanket
/// `'unsafe-inline'`: this origin also serves user-uploaded files, which makes
/// script injection the specific thing worth defending against.
static APP_CSP: LazyLock<String> = LazyLock::new(build_app_csp);

fn build_app_csp() -> String {
    let script_src = match inline_script_hash() {
        Some(hash) => format!("'self' '{hash}'"),
        None => {
            // Without the hash the SPA cannot boot, so serve a working app
            // under a weaker policy rather than a blank page under a strict one.
            eprintln!(
                "WARN: could not hash the SPA bootstrap script; CSP will permit inline scripts"
            );
            "'self' 'unsafe-inline'".to_string()
        }
    };

    [
        "default-src 'self'".to_string(),
        format!("script-src {script_src}"),
        // Svelte writes dynamic `style=` attributes (meter widths, layout
        // sizing), which CSP counts as inline styles.
        "style-src 'self' 'unsafe-inline'".to_string(),
        "img-src 'self' data:".to_string(),
        "media-src 'self'".to_string(),
        "font-src 'self'".to_string(),
        "connect-src 'self'".to_string(),
        // The PDF preview frames a same-origin download URL.
        "frame-src 'self'".to_string(),
        "object-src 'none'".to_string(),
        "frame-ancestors 'none'".to_string(),
        "base-uri 'self'".to_string(),
        "form-action 'self'".to_string(),
    ]
    .join("; ")
}

/// SHA-256 over the text of the first `<script>` element in `index.html`,
/// formatted as a CSP source expression. The digest must cover exactly the
/// bytes between the tags, whitespace included, or the browser rejects it.
fn inline_script_hash() -> Option<String> {
    let index = crate::Asset::get("index.html")?;
    let html = std::str::from_utf8(&index.data).ok()?;

    let open = html.find("<script>")? + "<script>".len();
    let close = html[open..].find("</script>")? + open;

    let digest = Sha256::digest(html[open..close].as_bytes());
    Some(format!("sha256-{}", STANDARD.encode(digest)))
}

/// Baseline response headers for every route.
///
/// Headers a handler has already set are left alone: `download_file` serves
/// untrusted bytes and deliberately locks itself down harder than this.
pub async fn security_headers(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // Read before `next` takes ownership of the request.
    let tls = req
        .app_data::<web::Data<AppConfig>>()
        .is_some_and(|config| config.cookie_secure);

    let mut res = next.call(req).await?;
    let headers = res.headers_mut();

    if !headers.contains_key(header::CONTENT_SECURITY_POLICY)
        && let Ok(value) = HeaderValue::from_str(&APP_CSP)
    {
        headers.insert(header::CONTENT_SECURITY_POLICY, value);
    }

    if !headers.contains_key(header::X_FRAME_OPTIONS) {
        headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    }

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );

    // Gated on the operator's own "we are behind TLS" signal rather than on
    // `X-Forwarded-Proto`: a spoofed header would otherwise pin HSTS onto a
    // deployment that has no HTTPS to fall back to, locking users out.
    if tls {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000"),
        );
    }

    Ok(res.map_into_boxed_body())
}

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
