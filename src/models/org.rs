use std::fmt::Display;

use anyhow::Result;
use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

use crate::{
    models::{
        filter::{gen_request_search, Filter},
        user::UserInterface,
    },
    paginator::Paginator,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Status {
    #[serde(rename = "todo")]
    Todo,
    #[serde(rename = "raise")]
    Raise,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
    #[serde(rename = "pending")]
    Pending,
}

impl From<String> for Status {
    fn from(s: String) -> Self {
        match s.as_ref() {
            "todo" => Self::Todo,
            "raise" => Self::Raise,
            "success" => Self::Success,
            "failure" => Self::Failure,
            "pending" => Self::Pending,
            _ => Self::Todo,
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Status::Todo => "todo",
                Status::Raise => "raise",
                Status::Success => "success",
                Status::Failure => "failure",
                Status::Pending => "pending",
            }
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgInterface {
    pub id: i32,
    pub name: String,
    pub description1: Option<String>,
    pub description2: Option<String>,
    pub category: Option<String>,
    pub status: Option<Status>,
    #[serde(rename = "creationStamp")]
    pub creation_stamp: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInterface {
    pub id: i32,
    pub name: String,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    #[serde(rename = "zipCode")]
    pub zip_code: Option<String>,
    pub city: Option<String>,
    #[serde(rename = "creationStamp")]
    pub creation_stamp: NaiveDateTime,
}
pub struct Org(Pool);

impl Org {
    pub fn new(pool: Pool) -> Self {
        Org(pool)
    }

    pub async fn band_related_orgs_and_statuses(
        &self,
        id_user: i32,
        id_band: i32,
        filters: Vec<Filter>,
        paginator: Option<Paginator>,
    ) -> Result<(Vec<OrgInterface>, Paginator)> {
        let client = self.0.get().await?;
        let req_end = if !filters.is_empty() { " AND " } else { "" };
        let req_filter = gen_request_search(filters);
        let pag = if let Some(p) = paginator {
            p
        } else {
            Paginator::default()
        };

        let stmt = client
            .prepare_cached(
                format!(
                    "
                    SELECT
                        o.id,
                        o.name,
                        o.description,
                        a.description,
                        a.category,
                        oa.status,
                        o.creation_stamp
                    FROM org o
                    JOIN activity a ON a.id_org = o.id
                    LEFT JOIN org_assign oa ON oa.id_org = o.id
                    WHERE oa.id_user = $1 AND oa.id_band = $2 {}{}{}
                    ",
                    req_end, req_filter, pag,
                )
                .as_str(),
            )
            .await?;
        let rows = client
            .query(&stmt, &[&id_user, &id_band])
            .await?
            .iter()
            .map(|row| {
                let statst: Option<String> = row.get(5);
                let stat = statst.map(Status::from);

                OrgInterface {
                    id: row.get(0),
                    name: row.get(1),
                    description1: row.get(2),
                    description2: row.get(3),
                    category: row.get(4),
                    status: stat,
                    creation_stamp: row.get(6),
                }
            })
            .collect();
        let stmt = client
            .prepare_cached(
                "
                SELECT COUNT(o.id)
                FROM org o
                JOIN activity a ON a.id_org = o.id
                LEFT JOIN org_assign oa ON oa.id_org = o.id
                    WHERE oa.id_user = $1 AND oa.id_band = $2
        ",
            )
            .await?;
        let result = client.query(&stmt, &[&id_user, &id_band]).await?;
        let count: i32 = result[0].get(0);
        Ok((
            rows,
            Paginator {
                page: pag.page,
                size: pag.size,
                page_count: Some(count / pag.size),
                item_count: Some(count),
            },
        ))
    }

    pub async fn tag_org(
        &self,
        id_user: i32,
        id_band: i32,
        id_org: i32,
        status: Status,
    ) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "SELECT id FROM org_assign WHERE id_user = $1 AND id_band = $2 AND id_org = $3",
            )
            .await?;
        let rows = client.query(&stmt, &[&id_user, &id_band, &id_org]).await?;

        let stmt = if rows.is_empty() {
            client
                .prepare_cached(
                    "
                INSERT INTO org_assign(id_org, id_user, id_band, status) 
                VALUES ($1, $2, $3, $4)",
                )
                .await?
        } else {
            client
                .prepare_cached(
                    "
                UPDATE org_assign SET status = $4 
                WHERE id_org = $1 AND id_user = $2 AND id_band = $3
            ",
                )
                .await?
        };
        client
            .query(&stmt, &[&id_org, &id_user, &id_band, &status.to_string()])
            .await?;

        Ok(())
    }

    pub async fn get_categories(&self) -> Result<Vec<String>> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare("SELECT DISTINCT category FROM activity WHERE category IS NOT NULL")
            .await?;
        Ok(client
            .query(&stmt, &[])
            .await?
            .iter()
            .map(|row| {
                let res: String = row.get(0);
                res
            })
            .collect::<Vec<String>>())
    }

    pub async fn get_assigned_users(&self, id_band: i32) -> Result<Vec<UserInterface>> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
            SELECT u.id, u.pseudo, u.name, u.firstname, u.email, u.creation_stamp, u.last_login, u.verified,
            FROM cnm_user u JOIN org_assign oa ON oa.id_user = u.id 
            WHERE oa.id_band = $1
        ",
            )
            .await?;
        Ok(client
            .query(&stmt, &[&id_band])
            .await?
            .iter()
            .map(|row| UserInterface {
                id: row.get(0),
                pseudo: row.get(1),
                name: row.get(2),
                firstname: row.get(3),
                email: row.get(4),
                creation_stamp: row.get(5),
                last_login: row.get(7),
                verified: row.get(8),
                is_admin: None,
            })
            .collect::<Vec<UserInterface>>())
    }

    pub async fn get_contacts(&self, id_org: i32, id_band: i32) -> Result<Vec<ContactInterface>> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
                SELECT 
                    id, 
                    name, 
                    firstname, 
                    email, 
                    phone, 
                    address, 
                    zip_code, 
                    city, 
                    creation_stamp
                FROM contact
                WHERE id_org = $1 AND id_band = $2
            ",
            )
            .await?;
        Ok(client
            .query(&stmt, &[&id_org, &id_band])
            .await?
            .iter()
            .map(|row| ContactInterface {
                id: row.get(0),
                name: row.get(1),
                first_name: row.get(2),
                email: row.get(3),
                phone: row.get(4),
                address: row.get(5),
                zip_code: row.get(6),
                city: row.get(7),
                creation_stamp: row.get(8),
            })
            .collect())
    }

    pub async fn add_contact(
        &self,
        id_org: i32,
        id_band: i32,
        contact: ContactInterface,
    ) -> Result<i32> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "INSERT INTO contact(
                    id_org,
                    name,
                    firstname,
                    email,
                    phone,
                    address,
                    zip_code,
                    city,
                    id_band
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
            )
            .await?;
        let rows = client
            .query(
                &stmt,
                &[
                    &id_org,
                    &contact.name,
                    &contact.first_name,
                    &contact.email,
                    &contact.phone,
                    &contact.address,
                    &contact.zip_code,
                    &contact.city,
                    &id_band,
                ],
            )
            .await?;
        Ok(rows[0].get(0))
    }

    pub async fn update_contact(&self, contact: ContactInterface) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client.prepare_cached(
            "UPDATE contact
            SET name = $1, firstname = $2, email = $3, phone = $4, address = $5, zip_code = $6, city = $7"
        ).await?;
        client
            .query(
                &stmt,
                &[
                    &contact.name,
                    &contact.first_name,
                    &contact.email,
                    &contact.phone,
                    &contact.address,
                    &contact.zip_code,
                    &contact.city,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn remove_contact(&self, id_contact: i32) -> Result<()> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("DELETE FROM contact WHERE id = $1")
            .await?;
        client.query(&stmt, &[&id_contact]).await?;
        Ok(())
    }

    pub async fn get_contact_band_id(&self, id_contact: i32) -> Result<i32> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("SELECT id_band FROM contact WHERE id = $1")
            .await?;
        let res = client.query(&stmt, &[&id_contact]).await?;
        Ok(res[0].get(0))
    }
}