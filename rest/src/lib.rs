use async_trait::async_trait;
use negotiated::Responder;
use serde::Deserialize;

pub trait DbFactory<Connection>
where Connection: diesel::connection::SimpleConnection {
    fn db(&self) -> Connection;
}

#[async_trait]
pub trait RestCollection<QueryParameters, D, Connection>: Sized 
where D: Sized + Send + negotiated::HandlebarsFactory, Connection: diesel::connection::Connection
{
    async fn get(app_data: actix_web::web::Data<D>, query_parameters: actix_web::web::Query<QueryParameters>) -> Responder<D>;
}
#[async_trait]
pub trait Rest<T, NewT, D, Connection>: Sized 
where D: Sized + Send + negotiated::HandlebarsFactory, Connection: diesel::connection::Connection
{
    async fn post(app_data: actix_web::web::Data<D>, new_object: actix_web::web::Json<NewT>) -> Responder<D>;
    async fn get(app_data: actix_web::web::Data<D>, id: actix_web::web::Path<i32>) -> Responder<D>;
    async fn put(app_data: actix_web::web::Data<D>, id: actix_web::web::Path<i32>, to_update: actix_web::web::Json<T>) -> Responder<D>;
    async fn delete(app_data: actix_web::web::Data<D>, id: actix_web::web::Path<i32>) -> Responder<D>;
}
pub trait RestPre<T, NewT, D>
where D: Sized + Send + negotiated::HandlebarsFactory
{
    fn pre_post(app_data: &actix_web::web::Data<D>, new_object: &NewT) -> anyhow::Result<NewT>;
    fn pre_get(app_data: &actix_web::web::Data<D>, id: i32) -> anyhow::Result<i32>;
    fn pre_put(app_data: &actix_web::web::Data<D>, id: i32, to_update: &T) -> anyhow::Result<(i32, T)>;
    fn pre_delete(app_data: &actix_web::web::Data<D>, id: i32) -> anyhow::Result<i32>;
}
pub trait RestPost<T, NewT>: Sized {
    fn post_post(new_object: &NewT, result: anyhow::Result<T>) -> anyhow::Result<T>;
    fn post_get(id: i32, result: anyhow::Result<T>) -> anyhow::Result<T>;
    fn post_put(id: i32, to_update: &T, result: anyhow::Result<T>) -> anyhow::Result<T>;
    fn post_delete(id: i32, result: anyhow::Result<T>) -> anyhow::Result<T>;
}
pub trait Crud<T, NewT, Connection>
where Connection: diesel::connection::Connection
{
    fn db_insert(db: &mut Connection, to_insert: &NewT) -> anyhow::Result<T>;
    fn db_update(db: &mut Connection, to_update: &T) -> anyhow::Result<T>;
    fn db_fetch_all(db: &mut Connection, name_filter: String, limit: Option<(i64, i64)>) -> anyhow::Result<Vec<T>>;
    fn db_fetch(db: &mut Connection, id: i32) -> anyhow::Result<T>;
    fn db_delete(db: &mut Connection, id: i32) -> anyhow::Result<T>;
}
#[derive(Deserialize)]
pub struct RestCollectionGetParameters {
    pub q: Option<String>
}
