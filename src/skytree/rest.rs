use async_trait::async_trait;

use crate::negotiated::Responder;

#[async_trait]
pub trait RestCollection: Sized {
    async fn get() -> Responder;
}