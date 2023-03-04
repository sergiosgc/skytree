use std::env;

use async_trait::async_trait;
use crate::negotiated::Responder;
use serde::{Deserialize, Serialize};
use super::rest::{RestCollection, Rest};
use diesel::{self, *};
use crate::schema;

#[derive(Debug,Clone, Serialize, Deserialize, Queryable, AsChangeset)]
#[diesel(table_name = schema::host_group)]
pub struct HostGroup {
    pub id: i32,
    pub parent: Option<i32>,
    pub name: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = schema::host_group)]
pub struct NewHostGroup {
    pub parent: Option<i32>,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct RestCollectionGetParameters {
    pub q: Option<String>
}
#[async_trait]
impl RestCollection<RestCollectionGetParameters> for HostGroup {
    async fn get(actix_web::web::Query(query_parameters): actix_web::web::Query<RestCollectionGetParameters>) -> Responder {
        HostGroup::db_fetch_all(query_parameters.q.unwrap_or_default(), None).await.into()
    }
}
#[async_trait]
impl Rest<HostGroup, NewHostGroup> for HostGroup {
    async fn post(actix_web::web::Json(host_group): actix_web::web::Json<NewHostGroup>) -> Responder {
        HostGroup::db_insert(&host_group).await.into()
    }
    async fn get(id: actix_web::web::Path<u32>) -> Responder {
        HostGroup::db_fetch(id.into_inner().try_into().unwrap()).await.into()
    }
    async fn put(id: actix_web::web::Path<u32>, to_update: actix_web::web::Json<HostGroup>) -> Responder {
        let mut merged_to_update = to_update.clone();
        merged_to_update.id = id.into_inner().try_into().unwrap();
        HostGroup::db_update(&merged_to_update).await.into()
    }
    async fn delete(id: actix_web::web::Path<u32>) -> Responder {
        HostGroup::db_delete(id.into_inner().try_into().unwrap()).await.into()
    }
}
impl HostGroup {
    fn db() -> SqliteConnection {
        SqliteConnection::establish(&env::var("DATABASE_URL").expect("DATABASE_URL must be set")).unwrap()
    }
    pub async fn db_insert(to_insert: &NewHostGroup) -> anyhow::Result<HostGroup> {
        let mut conn = HostGroup::db();
        Ok(diesel::insert_into(schema::host_group::table)
            .values(to_insert)
            .get_result::<HostGroup>(&mut conn)?)
    }
    pub async fn db_update(to_update: &HostGroup) -> anyhow::Result<HostGroup> {
        let mut conn = HostGroup::db();
        Ok(diesel::update(schema::host_group::table)
            .filter(crate::schema::host_group::dsl::id.eq(to_update.id))
            .set(to_update)
            .get_result::<HostGroup>(&mut conn)?)
    }
    pub async fn db_fetch_all(name_filter: String, limit: Option<(i64, i64)>) -> anyhow::Result<Vec<HostGroup>> {
        let mut conn = HostGroup::db();
        let mut query = schema::host_group::dsl::host_group.into_boxed();
        if !name_filter.is_empty() { query = query.filter(crate::schema::host_group::dsl::name.like(name_filter)); }
        if limit.is_some() { query = query.limit(limit.unwrap().0).offset(limit.unwrap().1); }
        Ok(query.load::<HostGroup>(&mut conn)?)
    }
    pub async fn db_fetch(id: i32) -> anyhow::Result<HostGroup> {
        let mut conn = HostGroup::db();
        Ok(schema::host_group::dsl::host_group
            .filter(crate::schema::host_group::dsl::id.eq(id))
            .first(&mut conn)?)
    }
    pub async fn db_delete(id: i32) -> anyhow::Result<HostGroup> {
        let mut conn = HostGroup::db();
        let result = Self::db_fetch(id).await;
        diesel::delete(schema::host_group::table)
            .filter(crate::schema::host_group::dsl::id.eq(id))
            .execute(&mut conn)?;
        result
    }
}