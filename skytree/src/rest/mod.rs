use async_trait::async_trait;
use diesel::SqliteConnection;

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
pub trait Crud<T, NewT> {
    fn db() -> SqliteConnection;
    fn db_insert(to_insert: &NewT) -> anyhow::Result<T>;
    fn db_update(to_update: &T) -> anyhow::Result<T>;
    fn db_fetch_all(name_filter: String, limit: Option<(i64, i64)>) -> anyhow::Result<Vec<T>>;
    fn db_fetch(id: i32) -> anyhow::Result<T>;
    fn db_delete(id: i32) -> anyhow::Result<T>;
}
pub trait SimpleRest<T, NewT>: Sized {}
/*
#[async_trait]
impl<T, NewT> Rest<T, NewT> for dyn SimpleRest<T, NewT>
    where T: Sized,
          NewT: Sized
{
    async fn post(actix_web::web::Json(t): actix_web::web::Json<NewT>) -> Responder {
        actix_web::web::block(move || -> Responder {
            T::db_insert(&t).into()
        }).await.into()
    }
    async fn get(id: actix_web::web::Path<u32>) -> Responder {
        actix_web::web::block(move || -> Responder {
            T::db_fetch(id.into_inner().try_into().unwrap()).into()
        }).await.into()
    }
    async fn put(id: actix_web::web::Path<u32>, to_update: actix_web::web::Json<T>) -> Responder {
        actix_web::web::block(move || -> Responder {
            let mut merged_to_update = to_update.clone();
            merged_to_update.id = id.into_inner().try_into().unwrap();
            T::db_update(&merged_to_update).into()
        }).await.into()
    }
    async fn delete(id: actix_web::web::Path<u32>) -> Responder {
        actix_web::web::block(move || -> Responder {
            T::db_delete(id.into_inner().try_into().unwrap()).into()
        }).await.into()
    }
}
*/