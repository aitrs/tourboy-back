use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

use crate::models::band::BandInterface;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInterface {
    pub id: i32,
    pub pseudo: String,
    pub name: String,
    pub firstname: String,
    pub email: String,
    #[serde(rename = "creationStamp")]
    pub creation_stamp: NaiveDateTime,
    #[serde(rename = "lastLogin")]
    pub last_login: Option<NaiveDateTime>,
    pub verified: bool,
    #[serde(rename = "isAdmin")]
    pub is_admin: Option<bool>,
}

#[derive(Clone)]
pub struct User(Pool);

impl User {
    pub fn new(pool: Pool) -> Self {
        User(pool)
    }

    pub async fn exists(&self, email: String) -> Result<(bool, Option<UserInterface>)> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
            SELECT
                cu.id,
                cu.pseudo,
                cu.name,
                cu.firstname,
                cu.email,
                cu.creation_stamp,
                cu.last_login,
                cu.verified
            FROM cnm_user cu
            WHERE cu.email = $1 AND cu.verified IS TRUE;
        ",
            )
            .await?;
        let res = client
            .query(&stmt, &[&email])
            .await?
            .iter()
            .map(|r| UserInterface {
                id: r.get(0),
                pseudo: r.get(1),
                name: r.get(2),
                firstname: r.get(3),
                email: r.get(4),
                creation_stamp: r.get(5),
                last_login: r.get(6),
                verified: r.get(7),
                is_admin: None,
            })
            .collect::<Vec<UserInterface>>();

        if !res.is_empty() {
            Ok((true, Some(res[0].clone())))
        } else {
            Ok((false, None))
        }
    }

    pub async fn create(
        &self,
        pseudo: String,
        email: String,
        name: String,
        firstname: String,
        pwd: String,
    ) -> Result<i32> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "INSERT INTO cnm_user(pseudo, name, firstname, email, pwd)
            VALUES ($1, $2, $3, $4, crypt($5, gen_salt('bf'))) RETURNING id",
            )
            .await?;
        let rows = client
            .query(&stmt, &[&pseudo, &name, &firstname, &email, &pwd])
            .await?;
        let id: i32 = rows[0].get(0);
        Ok(id)
    }

    pub async fn authenticate(&self, email: String, pwd: String) -> Result<bool> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("SELECT (pwd = crypt($1, pwd)) AS match FROM cnm_user WHERE email = $2")
            .await?;
        let rows = client.query(&stmt, &[&pwd, &email]).await?;
        if rows.is_empty() {
            Ok(false)
        } else {
            let matched: bool = rows[0].get(0);
            if matched {
                let stmt = client
                    .prepare_cached("UPDATE cnm_user SET last_login = CURRENT_TIMESTAMP")
                    .await?;
                client.query(&stmt, &[]).await?;
            }
            Ok(matched)
        }
    }

    pub async fn authenticate_with_id(&self, id_user: i32, pwd: String) -> Result<bool> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("SELECT (pwd = crypt($1, pwd)) AS match FROM cnm_user WHERE id = $2")
            .await?;
        let rows = client.query(&stmt, &[&pwd, &id_user]).await?;
        let matched: bool = rows[0].get(0);
        if matched {
            let stmt = client
                .prepare_cached("UPDATE cnm_user SET last_login = CURRENT_TIMESTAMP")
                .await?;
            client.query(&stmt, &[]).await?;
        }
        Ok(matched)
    }

    pub async fn update(&self, id: i32, field: String, value: String) -> Result<()> {
        if [
            "pseudo".to_string(),
            "name".to_string(),
            "firstname".to_string(),
            "email".to_string(),
            "pwd".to_string(),
        ]
        .contains(&field)
        {
            let client = self.0.get().await?;
            let stmt = if field == "pwd" {
                client
                    .prepare_cached("UPDATE cnm_user SET pwd=crypt($1, gen_salt('bf')) WHERE id=$2")
                    .await?
            } else {
                client
                    .prepare_cached(&format!("UPDATE cnm_user SET {}=$1 WHERE id=$2", field))
                    .await?
            };
            client.query(&stmt, &[&value, &id]).await?;
            Ok(())
        } else {
            Err(anyhow!(format!("Le champ {} n'existe pas", field)))
        }
    }

    pub async fn delete(&self, id: i32) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("DELETE FROM cnm_user WHERE id = $1")
            .await?;
        client.query(&stmt, &[&id]).await?;
        Ok(())
    }

    pub async fn read(&self, id: i32) -> Result<UserInterface> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "SELECT id, pseudo, name, firstname, email, creation_stamp, last_login, verified FROM cnm_user WHERE id= $1",
            )
            .await?;
        let rows: Vec<UserInterface> = client
            .query(&stmt, &[&id])
            .await?
            .iter()
            .map(|row| UserInterface {
                id: row.get(0),
                pseudo: row.get(1),
                name: row.get(2),
                firstname: row.get(3),
                email: row.get(4),
                creation_stamp: row.get(6),
                last_login: row.get(7),
                verified: row.get(8),
                is_admin: None,
            })
            .collect();

        Ok(rows[0].clone())
    }

    pub async fn get_id_from_email(&self, email: String) -> Result<Option<i32>> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("SELECT id FROM cnm_user WHERE email = $1")
            .await?;
        let rows = client.query(&stmt, &[&email]).await?;

        if rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(rows[0].get(0)))
        }
    }

    pub async fn add_band(&self, id_user: i32, id_band: i32, admin: bool) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("INSERT INTO user_band(id_user, id_band, is_admin) VALUES($1, $2, $3)")
            .await?;
        client.query(&stmt, &[&id_user, &id_band, &admin]).await?;
        Ok(())
    }

    pub async fn exit_band(&self, id_user: i32, id_band: i32) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("DELETE FROM user_band WHERE id_user = $1 AND id_band = $2")
            .await?;
        client.query(&stmt, &[&id_user, &id_band]).await?;
        let stmt = client
            .prepare_cached(
                "SELECT CAST(COUNT(id_user) as INT) FROM user_band WHERE id_band = $1 AND is_admin = true",
            )
            .await?;
        let rows = client.query(&stmt, &[&id_band]).await?;
        let count: i32 = rows[0].get(0);

        if count == 0 {
            let stmt = client
                .prepare_cached("DELETE FROM band WHERE id = $1")
                .await?;
            client.query(&stmt, &[&id_band]).await?;
        }

        Ok(())
    }

    pub async fn verify(&self, email: String, chain: String) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "UPDATE cnm_user SET verified = true WHERE email = $1 AND verify_chain = $2",
            )
            .await?;
        client.query(&stmt, &[&email, &chain]).await?;
        Ok(())
    }

    pub async fn deactivate(&self, id_user: i32) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("UPDATE cnm_user SET verified = false WHERE id = $1")
            .await?;
        client.query(&stmt, &[&id_user]).await?;
        Ok(())
    }

    pub async fn get_bands(&self, id_user: i32) -> Result<Vec<BandInterface>> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "SELECT b.id, b.name, b.creation_stamp 
            FROM band b 
            JOIN user_band ub ON b.id = ub.id_band 
            WHERE ub.id_user = $1",
            )
            .await?;
        let rows = client
            .query(&stmt, &[&id_user])
            .await?
            .iter()
            .map(|row| BandInterface {
                id: row.get(0),
                name: row.get(1),
                creation_stamp: row.get(2),
            })
            .collect();
        Ok(rows)
    }
}
