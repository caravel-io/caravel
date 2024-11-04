use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, Multipart, Request},
    http::{header::CONTENT_TYPE, StatusCode},
    Json,
};

use super::events::{Event, EventType};

struct Manifest(Bytes);

#[async_trait]
impl<S> FromRequest<S> for Manifest
where
    Bytes: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Some(content_type) = req.headers().get(CONTENT_TYPE) else {
            return Err(StatusCode::BAD_REQUEST);
        };

        let body = if content_type == "multipart/form-data" {
            let mut multipart = Multipart::from_request(req, state)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            let Ok(Some(field)) = multipart.next_field().await else {
                return Err(StatusCode::BAD_REQUEST);
            };

            field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?
        } else if content_type == "image/jpeg" {
            Bytes::from_request(req, state)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?
        } else {
            return Err(StatusCode::BAD_REQUEST);
        };

        Ok(Self(body))
    }
}

pub async fn handler() -> (StatusCode, Json<Event>) {
    let resp = Event::new(EventType::Assembled).message("received dependencies".to_lowercase());
    return (StatusCode::ACCEPTED, Json(resp));
}
