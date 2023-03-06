use std::env;

use async_trait::async_trait;
use crate::{negotiated::Responder, rest::{RestCollection, Rest, Crud, SimpleRest}};
use serde::{Deserialize, Serialize};
use diesel::{self, *};
use crate::schema;

#[derive(Debug,Clone, Serialize, Deserialize, Queryable, AsChangeset)]
#[diesel(table_name = schema::host_group)]
pub struct HostGroup {
    pub id: i32,
    pub parent: Option<i32>,
    pub name: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
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
        actix_web::web::block(move || -> Responder {
            HostGroup::db_fetch_all(query_parameters.q.unwrap_or_default(), None).into()
        }).await.into()
    }
}
impl SimpleRest<HostGroup, NewHostGroup> for HostGroup {}
#[async_trait]
impl Rest<HostGroup, NewHostGroup> for HostGroup {
    async fn post(actix_web::web::Json(host_group): actix_web::web::Json<NewHostGroup>) -> Responder {
        actix_web::web::block(move || -> Responder {
            HostGroup::db_insert(&host_group).into()
        }).await.into()
    }
    async fn get(id: actix_web::web::Path<u32>) -> Responder {
        actix_web::web::block(move || -> Responder {
            HostGroup::db_fetch(id.into_inner().try_into().unwrap()).into()
        }).await.into()
    }
    async fn put(id: actix_web::web::Path<u32>, to_update: actix_web::web::Json<HostGroup>) -> Responder {
        actix_web::web::block(move || -> Responder {
            let mut merged_to_update = to_update.clone();
            merged_to_update.id = id.into_inner().try_into().unwrap();
            HostGroup::db_update(&merged_to_update).into()
        }).await.into()
    }
    async fn delete(id: actix_web::web::Path<u32>) -> Responder {
        actix_web::web::block(move || -> Responder {
            HostGroup::db_delete(id.into_inner().try_into().unwrap()).into()
        }).await.into()
    }
}
impl Crud<HostGroup, NewHostGroup> for HostGroup {
    fn db() -> SqliteConnection {
        SqliteConnection::establish(&env::var("DATABASE_URL").expect("DATABASE_URL must be set")).unwrap()
    }
    fn db_insert(to_insert: &NewHostGroup) -> anyhow::Result<HostGroup> {
        let mut conn = HostGroup::db();
        Ok(diesel::insert_into(schema::host_group::table)
            .values(to_insert)
            .get_result::<HostGroup>(&mut conn)?)
    }
    fn db_update(to_update: &HostGroup) -> anyhow::Result<HostGroup> {
        let mut conn = HostGroup::db();
        Ok(diesel::update(schema::host_group::table)
            .filter(crate::schema::host_group::dsl::id.eq(to_update.id))
            .set(to_update)
            .get_result::<HostGroup>(&mut conn)?)
    }
    fn db_fetch_all(name_filter: String, limit: Option<(i64, i64)>) -> anyhow::Result<Vec<HostGroup>> {
        let mut conn = HostGroup::db();
        let mut query = schema::host_group::dsl::host_group.into_boxed();
        if !name_filter.is_empty() { query = query.filter(crate::schema::host_group::dsl::name.like(name_filter)); }
        if let Some(limit_value) = limit { query = query.limit(limit_value.0).offset(limit_value.1); }
        Ok(query.load::<HostGroup>(&mut conn)?)
    }
    fn db_fetch(id: i32) -> anyhow::Result<HostGroup> {
        let mut conn = HostGroup::db();
        Ok(schema::host_group::dsl::host_group
            .filter(crate::schema::host_group::dsl::id.eq(id))
            .first(&mut conn)?)
    }
    fn db_delete(id: i32) -> anyhow::Result<HostGroup> {
        let mut conn = HostGroup::db();
        let result = Self::db_fetch(id);
        diesel::delete(schema::host_group::table)
            .filter(crate::schema::host_group::dsl::id.eq(id))
            .execute(&mut conn)?;
        result
    }
}