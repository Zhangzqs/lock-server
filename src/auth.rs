use chrono::Utc;
use serde::Deserialize;
use sqlx::sqlite::{SqlitePool, SqliteQueryAs};

pub struct UserManager<'a> {
    pool: &'a SqlitePool,
}

pub(crate) type CardIdType = i64;

#[derive(sqlx::FromRow)]
pub struct UnlockLog {
    name: String,
    ts: i64,
}

#[derive(Debug, Default, Deserialize, sqlx::FromRow)]
pub struct User {
    /// Student id
    #[serde(rename = "studentId")]
    pub student_id: String,
    /// Student name
    pub name: String,
    /// Card id
    pub card: CardIdType,
    /// Create time
    pub created_at: Option<i64>,
}

impl User {
    /// Create a new user.
    pub fn new(student_id: String, name: String, card: CardIdType) -> User {
        User {
            student_id,
            name,
            card,
            created_at: Some(Utc::now().naive_local().timestamp()),
        }
    }

    pub async fn fetch_log(self, pool: &SqlitePool) -> anyhow::Result<Vec<UnlockLog>> {
        let log = sqlx::query_as(
            "SELECT name, unlock_at as ts FROM user, log
            WHERE user.card = log.card_id AND user.student_id = $1",
        )
        .bind(self.student_id)
        .fetch_all(pool)
        .await?;
        Ok(log)
    }

    pub async fn append_log(&self, pool: &SqlitePool, ts: i64) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO log (card_id, unlock_at) VALUES ($1, $2)")
            .bind(&self.student_id)
            .bind(ts)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl<'a> UserManager<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Query user basic information by card id.
    pub async fn query_by_card(&self, card_id: CardIdType) -> anyhow::Result<Option<User>> {
        let stu: Option<User> =
            sqlx::query_as("SELECT student_id, name, card, created_at FROM user WHERE card = $1")
                .bind(card_id)
                .fetch_optional(self.pool)
                .await?;
        Ok(stu)
    }

    /// Query user basic information by id.
    pub async fn query_by_student_id(&self, student_id: &str) -> anyhow::Result<Option<User>> {
        let stu: Option<User> =
            sqlx::query_as("SELECT student_id, name, card, created_at FROM user WHERE student_id = $1")
                .bind(student_id)
                .fetch_optional(self.pool)
                .await?;
        Ok(stu)
    }

    /// Append a new user
    pub async fn add(&self, new_user: User) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO user (student_id, name, card) VALUES ($1, $2, $3)")
            .bind(new_user.student_id)
            .bind(new_user.name)
            .bind(new_user.card)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// Remove an existed user.
    pub async fn remove(&self, student_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM user WHERE student_id = $1")
            .bind(student_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
