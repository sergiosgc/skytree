use async_trait::async_trait;

use crate::negotiated::Responder;

#[async_trait]
pub trait RestCollection<QueryParameters>: Sized {
    async fn get(query_parameters: actix_web::web::Query<QueryParameters>) -> Responder;
}
#[async_trait]
pub trait Rest<T, NewT>: Sized {
    async fn post(new_object: actix_web::web::Json<NewT>) -> Responder;
    async fn get(id: actix_web::web::Path<u32>) -> Responder;
    async fn put(id: actix_web::web::Path<u32>, to_update: actix_web::web::Json<T>) -> Responder;
    async fn delete(id: actix_web::web::Path<u32>) -> Responder;
}
