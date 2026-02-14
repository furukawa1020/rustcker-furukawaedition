use axum::{extract::Request, middleware::Next, response::Response};

use tracing::{info, info_span, Instrument};
use uuid::Uuid;

pub async fn trace_request(req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    
    // In a real implementation, we would extract 'traceparent' here.
    // For now, we start a new root span or use the parent if extracted.
    
    let span = info_span!(
        "http_request",
        request_id = %request_id,
        method = %req.method(),
        uri = %req.uri(),
    );

    async move {
        info!("Request started");
        let response = next.run(req).await;
        info!(status = %response.status(), "Request finished");
        response
    }
    .instrument(span)
    .await
}
