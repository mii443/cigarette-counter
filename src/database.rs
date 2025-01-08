use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPool, Error};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub discord_id: String,
    pub username: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmokingType {
    pub id: i32,
    pub type_name: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmokingLog {
    pub id: i32,
    pub discord_id: String,
    pub smoking_type_id: i32,
    pub quantity: i32,
    pub smoked_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySmokingSummary {
    pub discord_id: String,
    pub username: String,
    pub smoke_date: NaiveDate,

    pub type_name: String,
    pub description: String,
    pub total_quantity: Option<i64>,
}

pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    /// Creates a new Database instance.
    ///
    /// # Arguments
    /// * `pool` - The PostgreSQL connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    /// Creates a new user in the database.
    ///
    /// # Arguments
    /// * `discord_id` - The Discord ID of the user.
    /// * `username` - The username of the user.
    ///
    /// # Returns
    /// A Result containing the created `User` or an `Error`.
    pub async fn create_user(&self, discord_id: &str, username: &str) -> Result<User, Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (discord_id, username)
            VALUES ($1, $2)
            RETURNING 
                discord_id as "discord_id!", 
                username as "username!", 
                created_at, 
                updated_at
            "#,
            discord_id,
            username
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(user)
    }

    /// Gets an existing user or creates a new one if it doesn't exist.
    ///
    /// # Arguments
    /// * `discord_id` - The Discord ID of the user.
    /// * `username` - The username of the user.
    ///
    /// # Returns
    /// A Result containing the `User` or an `Error`.
    pub async fn get_or_create_user(
        &self,
        discord_id: &str,
        username: &str,
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await?;

        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                discord_id,
                username,
                created_at as "created_at!",
                updated_at as "updated_at!"
            FROM users
            WHERE discord_id = $1
            "#,
            discord_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let user = match user {
            Some(user) => {
                if user.username != username {
                    sqlx::query_as!(
                        User,
                        r#"
                        UPDATE users
                        SET username = $2, updated_at = CURRENT_TIMESTAMP
                        WHERE discord_id = $1
                        RETURNING 
                            discord_id,
                            username,
                            created_at as "created_at!",
                            updated_at as "updated_at!"
                        "#,
                        discord_id,
                        username
                    )
                    .fetch_one(&mut *tx)
                    .await?
                } else {
                    user
                }
            }
            None => {
                sqlx::query_as!(
                    User,
                    r#"
                    INSERT INTO users (discord_id, username)
                    VALUES ($1, $2)
                    RETURNING 
                        discord_id,
                        username,
                        created_at as "created_at!",
                        updated_at as "updated_at!"
                    "#,
                    discord_id,
                    username
                )
                .fetch_one(&mut *tx)
                .await?
            }
        };

        tx.commit().await?;

        Ok(user)
    }

    /// Checks if a user exists in the database.
    ///
    /// # Arguments
    /// * `discord_id` - The Discord ID of the user.
    ///
    /// # Returns
    /// A Result containing a boolean indicating whether the user exists or an `Error`.
    pub async fn user_exists(&self, discord_id: &str) -> Result<bool, Error> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE discord_id = $1) as "exists!"
            "#,
            discord_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(exists)
    }

    /// Logs a smoking event.
    ///
    /// # Arguments
    /// * `discord_id` - The Discord ID of the user.
    /// * `smoking_type_id` - The ID of the smoking type.
    /// * `quantity` - The quantity of cigarettes smoked.
    ///
    /// # Returns
    /// A Result containing the logged `SmokingLog` or an `Error`.
    pub async fn log_smoking(
        &self,
        discord_id: &str,

        smoking_type_id: i32,
        quantity: i32,
    ) -> Result<SmokingLog, Error> {
        let log = sqlx::query_as!(
            SmokingLog,
            r#"
            INSERT INTO smoking_logs (discord_id, smoking_type_id, quantity)
            VALUES ($1, $2, $3)

            RETURNING 
                id as "id!", 
                discord_id as "discord_id!", 
                smoking_type_id as "smoking_type_id!", 
                quantity as "quantity!",
                smoked_at as "smoked_at!",
                created_at,
                updated_at

            "#,
            discord_id,
            smoking_type_id,
            quantity
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(log)
    }

    /// Retrieves the daily smoking summary for a user.
    ///
    /// # Arguments
    /// * `discord_id` - The Discord ID of the user.
    /// * `date` - The date for which to retrieve the summary.
    ///
    /// # Returns
    /// A Result containing a vector of `DailySmokingSummary` or an `Error`.
    pub async fn get_daily_summary(
        &self,
        discord_id: &str,
        date: NaiveDate,
    ) -> Result<Vec<DailySmokingSummary>, Error> {
        let summary = sqlx::query_as!(
            DailySmokingSummary,
            r#"
            SELECT 
                sl.discord_id as "discord_id!",
                u.username as "username!",
                DATE(sl.smoked_at) as "smoke_date!",
                st.type_name as "type_name!",
                st.description as "description!",
                SUM(sl.quantity) as total_quantity
            FROM smoking_logs sl
            JOIN users u ON sl.discord_id = u.discord_id
            JOIN smoking_types st ON sl.smoking_type_id = st.id
            WHERE sl.discord_id = $1 
            AND DATE(sl.smoked_at) = $2
            GROUP BY 
                sl.discord_id,
                u.username,
                DATE(sl.smoked_at),
                st.type_name,
                st.description
            "#,
            discord_id,
            date
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(summary)
    }

    /// Retrieves a smoking type by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the smoking type.
    ///
    /// # Returns
    /// A Result containing the `SmokingType` or an `Error`.
    pub async fn get_smoking_type(&self, id: i32) -> Result<SmokingType, Error> {
        let smoking_type = sqlx::query_as!(
            SmokingType,
            r#"
            SELECT 
                id as "id!", 
                type_name as "type_name!", 
                description,
                created_at
            FROM smoking_types
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(smoking_type)
    }

    /// Retrieves all smoking types.
    ///
    /// # Returns
    /// A Result containing a vector of `SmokingType` or an `Error`.
    pub async fn get_smoking_types(&self) -> Result<Vec<SmokingType>, Error> {
        let types = sqlx::query_as!(
            SmokingType,
            r#"
            SELECT 
                id as "id!",
                type_name as "type_name!",
                description,
                created_at
            FROM smoking_types
            ORDER BY id
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(types)
    }

    /// Checks if a smoking type exists in the database.
    ///
    /// # Arguments
    /// * `id` - The ID of the smoking type.
    ///
    /// # Returns
    /// A Result containing a boolean indicating whether the smoking type exists or an `Error`.
    pub async fn smoking_type_exists(&self, id: i32) -> Result<bool, Error> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(SELECT 1 FROM smoking_types WHERE id = $1) as "exists!"
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(exists)
    }
}
