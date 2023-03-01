use std::{collections::HashMap, str::FromStr};
use async_trait::async_trait;
use sqlx::{sqlite::{SqliteConnectOptions, SqliteJournalMode}, ConnectOptions};
use crate::negotiated::Responder;
use serde::{Deserialize, Serialize};

use super::{rest::RestCollection};

#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct HostGroup {
    pub id: i64,
    pub parent: Option<i64>,
    pub name: Option<String>,
}
#[async_trait]
impl RestCollection for HostGroup {
    async fn get() -> Responder {
        crate::negotiated::Responder { payload: Some(Box::new(HostGroup::retrieve_all(HashMap::new()).await)), ..Default::default() }
    }
}
impl HostGroup {
    pub async fn retrieve_all(filter: HashMap<String, String>) -> Vec<HostGroup> {
        let mut conn = SqliteConnectOptions::from_str("sqlite://database.sqlite")
            .unwrap()
            .journal_mode(SqliteJournalMode::Wal)
            .read_only(false)
            .connect().await.unwrap();
        sqlx::query_as!(HostGroup, r#"
SELECT 
 "id",
 "parent",
 "name"
FROM
 "host_group"
ORDER BY "id"
        "#)
        .fetch_all(&mut conn)
        .await
        .unwrap()


    }
}