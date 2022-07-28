use anyhow::Result;
use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

use super::user::UserInterface;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandInterface {
    pub id: i32,
    pub name: String,
    pub creation_stamp: NaiveDateTime,
}
pub struct Band(Pool);

impl Band {
    pub fn new(pool: Pool) -> Self {
        Band(pool)
    }

    pub async fn create(&self, id_user: i32, name: String) -> Result<i32> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("INSERT INTO band(name, id_creator) VALUES ($1, $2) RETURNING id")
            .await?;
        let rows = client.query(&stmt, &[&name, &id_user]).await?;
        let id: i32 = rows[0].get(0);
        let stmt = client
            .prepare_cached("INSERT INTO user_band(id_user, id_band, is_admin) VALUES ($1, $2, true)")
            .await?;
        client.query(&stmt, &[&id_user, &id]).await?;
        Ok(id)
    }

    pub async fn remove(&self, id: i32) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("DELETE FROM band WHERE id = $1 CASCADE")
            .await?;
        client.query(&stmt, &[&id]).await?;
        Ok(())
    }

    pub async fn edit(&self, id: i32, name: String) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("UPDATE band SET name = $1 WHERE id = $2")
            .await?;
        client.query(&stmt, &[&name, &id]).await?;
        Ok(())
    }

    pub async fn is_admin(&self, id_user: i32, id_band: i32) -> Result<bool> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("SELECT is_admin FROM user_band WHERE id_user = $1 AND id_band = $2")
            .await?;
        let rows = client.query(&stmt, &[&id_user, &id_band]).await?;
        Ok(rows[0].get(0))
    }

    pub async fn get_admin_count(&self, id_band: i32) -> Result<i32> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "SELECT COUNT(id_user) FROM user_band WHERE id_band = $1 AND is_admin = true",
            )
            .await?;
        let rows = client.query(&stmt, &[&id_band]).await?;
        Ok(rows[0].get(0))
    }

    pub async fn get_band_members(&self, id_band: i32) -> Result<Vec<UserInterface>> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("
                SELECT
                    cu.id,
                    cu.pseudo,
                    cu.name,
                    cu.firstname,
                    cu.email,
                    cu.creation_stamp,
                    cu.last_login,
                    cu.verified,
                    ub.is_admin
                FROM cnm_user cu
                JOIN user_band ub 
                ON ub.id_user = cu.id
                WHERE ub.id_band = $1
            ").await?;
        let rows = client.query(&stmt, &[&id_band]).await?.iter().map(|row| UserInterface {
            id: row.get(0),
            pseudo: row.get(1),
            name: row.get(2),
            firstname: row.get(3),
            email: row.get(4),
            creation_stamp: row.get(5),
            last_login: row.get(6),
            verified: row.get(7),
            is_admin: Some(row.get(8)),
        }).collect();

        Ok(rows)
    }
}
