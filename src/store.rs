use async_trait::async_trait;
use sqlx::SqlitePool;
use std::{fmt::Display, sync::Arc};
use thiserror::Error;

use crate::item::{Item, State};

#[async_trait]
pub trait Store<T> {
    async fn create(&self, record: T) -> Result<i64, StoreError>;
    async fn delete(&self, id: i64) -> Result<(), StoreError>;
    async fn update(&self, record: T) -> Result<(), StoreError>;
    async fn read(&self, id: i64) -> Result<T, StoreError>;
    async fn read_many_from_state(&self, state: State) -> Result<Vec<T>, StoreError>;
}

#[derive(Error, Debug)]
pub enum StoreError {
    SqlError(#[from] sqlx::Error),
}

impl Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SqlError(error) => write!(f, "SqlError: {error}"),
        }
    }
}

#[derive(Clone)]
pub struct SqliteItemStore {
    pool: SqlitePool,
}

#[async_trait]
impl Store<Item> for SqliteItemStore {
    async fn create(&self, record: Item) -> Result<i64, StoreError> {
        let state = record.state as i64;
        // Insert the task, then obtain the ID of this row
        let query = sqlx::query!(
            r#"INSERT INTO item ( name, quantity, state ) VALUES (?1, ?2, ?3)"#,
            record.name,
            record.quantity,
            state,
        );
        let id = query.execute(&self.pool).await?.last_insert_rowid();

        Ok(id)
    }

    async fn delete(&self, id: i64) -> Result<(), StoreError> {
        // Insert the task, then obtain the ID of this row
        sqlx::query!(r#"DELETE FROM item WHERE id = ?1"#, id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update(&self, record: Item) -> Result<(), StoreError> {
        let state = record.state as i64;
        // Insert the task, then obtain the ID of this row
        sqlx::query!(
            r#"UPDATE item SET name = ?1, quantity = ?2, state = ?3 WHERE id = ?4"#,
            record.name,
            record.quantity,
            state,
            record.id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(())
    }

    async fn read(&self, id: i64) -> Result<Item, StoreError> {
        // Insert the task, then obtain the ID of this row
        let record = sqlx::query!(
            r#"SELECT id, name, quantity, state FROM item WHERE id = ?1"#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Item {
            id,
            name: record.name,
            quantity: record.quantity,
            state: record.state.into(),
        })
    }

    async fn read_many_from_state(&self, state: State) -> Result<Vec<Item>, StoreError> {
        let state = state as i64;
        let records = sqlx::query_as!(
            Item,
            r#"SELECT id, name, quantity, state FROM item WHERE state = ?1"#,
            state
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}

impl SqliteItemStore {
    pub async fn new(dsn: &str) -> Self {
        let pool = SqlitePool::connect(dsn)
            .await
            .inspect_err(|err| tracing::error!("{err} - dsn {dsn}"))
            .expect("SqlitePool for ItemStore couldn't be initialized");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("migrations failed to be executed");

        Self { pool }
    }
}
pub type ItemStore = Arc<dyn Store<Item> + Send + Sync>;
