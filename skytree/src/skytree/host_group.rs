use async_trait::async_trait;
use rest::{RestCollection, Rest, Crud};
use rest_derive::{Rest, Crud};
use serde::{Deserialize, Serialize};
use diesel::{self, *};
use crate::schema;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, AsChangeset, Rest, Crud)]
#[diesel(table_name = schema::host_group)]
#[rest(post=true,pre=true,app_data=crate::AppData<'static>,connection=diesel::sqlite::SqliteConnection)]
#[crud(table_name=schema::host_group, connection=diesel::sqlite::SqliteConnection)]
pub struct HostGroup {
    pub id: i32,
    pub parent: Option<i32>,
    pub name: Option<String>,
}