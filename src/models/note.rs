use anyhow::Result;
use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

use super::user::UserInterface;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInterface {
    pub id: i32,
    pub note: String,
    pub user: Option<UserInterface>,
    #[serde(rename = "creationStamp")]
    pub creation_stamp: NaiveDateTime,
}

pub struct Note(Pool);

impl Note {
    pub fn new(pool: Pool) -> Self {
        Note(pool)
    }

    pub async fn create(
        &self,
        id_user: i32,
        id_band: i32,
        id_activity: i32,
        note: String,
    ) -> Result<NoteInterface> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
                INSERT INTO note(id_user, id_band, id_activity, note) 
                VALUES($1, $2, $3, $4)
                RETURNING id, note, creation_stamp
            ",
            )
            .await?;
        let res = client
            .query(&stmt, &[&id_user, &id_band, &id_activity, &note])
            .await?;
        Ok(NoteInterface {
            id: res[0].get(0),
            note: res[0].get(1),
            user: None,
            creation_stamp: res[0].get(2),
        })
    }

    pub async fn edit(&self, id: i32, id_user: i32, note: String) -> Result<NoteInterface> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
                UPDATE note SET note = $1 WHERE id = $2 AND id_user = $3
                RETURNING id, note, creation_stamp
            ",
            )
            .await?;
        let res = client.query(&stmt, &[&note, &id, &id_user]).await?;

        Ok(NoteInterface {
            id: res[0].get(0),
            note: res[0].get(1),
            user: None,
            creation_stamp: res[0].get(2),
        })
    }

    pub async fn delete(&self, id: i32, id_user: i32) -> Result<Option<NoteInterface>> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
                DELETE FROM note WHERE id = $1 AND id_user = $2
                RETURNING id, note, creation_stamp
            ",
            )
            .await?;
        let res = client.query(&stmt, &[&id, &id_user]).await?;

        if res.is_empty() {
            Ok(None)
        } else {
            Ok(Some(NoteInterface {
                id: res[0].get(0),
                note: res[0].get(1),
                user: None,
                creation_stamp: res[0].get(2),
            }))
        }
    }

    pub async fn read_all(&self, id_activity: i32, id_band: i32) -> Result<Vec<NoteInterface>> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
                SELECT 
                    n.id, n.note, n.creation_stamp,
                    cu.id, cu.pseudo, cu.name, cu.firstname,
                    cu.email, cu.creation_stamp, cu.last_login,
                    cu.verified
                FROM note n
                JOIN cnm_user cu ON cu.id = n.id_user
                WHERE n.id_activity = $1 AND n.id_band = $2
            ",
            )
            .await?;
        Ok(client
            .query(&stmt, &[&id_activity, &id_band])
            .await?
            .iter()
            .map(|r| NoteInterface {
                id: r.get(0),
                note: r.get(1),
                creation_stamp: r.get(2),
                user: Some(UserInterface {
                    id: r.get(3),
                    pseudo: r.get(4),
                    name: r.get(5),
                    firstname: r.get(6),
                    email: r.get(7),
                    creation_stamp: r.get(8),
                    last_login: r.get(9),
                    verified: r.get(10),
                    is_admin: None,
                }),
            })
            .collect())
    }
}
