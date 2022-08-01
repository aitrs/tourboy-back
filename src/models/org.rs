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
    #[serde(rename = "idActivity")]
    pub id_activity: i32,
    #[serde(rename = "idOrg")]
    pub id_org: i32,
    pub name: String,
    pub description1: Option<String>,
    pub description2: Option<String>,
    pub category: Option<String>,
    pub city: Option<String>,
    #[serde(rename = "zipCode")]
    pub zip_code: Option<String>,
    pub status: Option<Status>,
    #[serde(rename = "userId")]
    pub user_id: Option<i32>,
    #[serde(rename = "userPseudo")]
    pub user_pseudo: Option<String>,
    #[serde(rename = "creationStamp")]
    pub creation_stamp: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgRawInterface {
    #[serde(rename = "idActivity")]
    pub id_activity: i32,
    #[serde(rename = "idOrg")]
    pub id_org: i32,
    pub name: String,
    pub description1: Option<String>,
    pub description2: Option<String>,
    pub category: Option<String>,
    pub city: Option<String>,
    #[serde(rename = "zipCode")]
    pub zip_code: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<i32>,
    #[serde(rename = "userPseudo")]
    pub user_pseudo: Option<String>,
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

    pub async fn all_orgs(
        &self,
        id_band: i32,
        filters: Vec<Filter>,
        paginator: Option<Paginator>,
    ) -> Result<(Vec<OrgRawInterface>, Paginator)> {
        let client = self.0.get().await?;
        let req_end = if !filters.is_empty() { " AND " } else { "" };
        let req_filter = gen_request_search(filters);
        let pag = if let Some(p) = paginator {
            p
        } else {
            Paginator::default()
        };
        let streq = format!(
            "
            SELECT
                a.id,
                o.name,
                o.description,
                a.description,
                a.city,
                a.postal_code,
                a.category,
                CAST(oa.status AS VARCHAR(16)) as status,
                cu.id,
                cu.pseudo,
                o.creation_stamp,
                o.id
            FROM org o
            JOIN activity a ON a.id_org = o.id
            LEFT JOIN org_assign oa ON oa.id_org = o.id
            LEFT JOIN cnm_user cu ON cu.id = oa.id_user
            WHERE (oa.id_band IS NULL OR oa.id_band = $1)
            {}{}{}
            ",
            req_end, req_filter, pag,
        );
        let stmt = client.prepare_cached(&streq).await?;
        let rows = client
            .query(&stmt, &[&id_band])
            .await?
            .iter()
            .map(|row| {
                let statst: Option<String> = row.get(7);
                OrgRawInterface {
                    id_activity: row.get(0),
                    name: row.get(1),
                    description1: row.get(2),
                    description2: row.get(3),
                    city: row.get(4),
                    zip_code: row.get(5),
                    category: row.get(6),
                    status: statst,
                    user_id: row.get(8),
                    user_pseudo: row.get(9),
                    creation_stamp: row.get(10),
                    id_org: row.get(11),
                }
            })
            .collect();
        let stmt = client
            .prepare_cached(
                format!(
                    "
                SELECT CAST(COUNT(o.id) AS INT)
                FROM org o
                JOIN activity a ON a.id_org = o.id
                LEFT JOIN org_assign oa ON oa.id_org = o.id
                LEFT JOIN cnm_user cu ON cu.id = oa.id_user
                WHERE (oa.id_band IS NULL OR oa.id_band = $1)
                {}{}
            ",
                    req_end, req_filter
                )
                .as_str(),
            )
            .await?;
        let result = client.query(&stmt, &[&id_band]).await?;
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

    pub async fn band_related_orgs_and_statuses(
        &self,
        id_user: i32,
        id_band: i32,
        filters: Vec<Filter>,
        paginator: Option<Paginator>,
    ) -> Result<(Vec<OrgRawInterface>, Paginator)> {
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
                        a.city,
                        a.postal_code,
                        a.category,
                        oa.status,
                        cu.id,
                        cu.pseudo,
                        o.creation_stamp,
                        o.id
                    FROM org o
                    JOIN activity a ON a.id_org = o.id
                    JOIN org_assign oa ON oa.id_org = o.id
                    JOIN cnm_user cu ON cu.id = oa.id_user,
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
                let statst: Option<String> = row.get(7);

                OrgRawInterface {
                    id_activity: row.get(0),
                    name: row.get(1),
                    description1: row.get(2),
                    description2: row.get(3),
                    city: row.get(4),
                    zip_code: row.get(5),
                    category: row.get(6),
                    status: statst,
                    user_id: row.get(8),
                    user_pseudo: row.get(9),
                    creation_stamp: row.get(10),
                    id_org: row.get(11),
                }
            })
            .collect();
        let stmt = client
            .prepare_cached(
                "
                SELECT CAST(COUNT(o.id) AS INT)
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

    pub async fn tag_orgs(
        &self,
        id_user: i32,
        id_band: i32,
        orgs: Vec<i32>,
        status: Status,
    ) -> Result<()> {
        let client = self.0.get().await?;
        let stmt1 = client
            .prepare_cached(
                "
                DELETE FROM org_assign WHERE id_org = $1 AND id_band = $2
            ",
            )
            .await?;
        let stmt2 = client
            .prepare_cached(
                format!(
                    "
                        INSERT INTO org_assign(id_org, id_user, id_band, status) 
                        VALUES ($1, $2, $3, '{}')
                    ",
                    status,                    
                ).as_str()
            )
            .await?;
        for id_org in orgs {
            client.query(&stmt1, &[&id_org, &id_band]).await?;
            client
                .query(&stmt2, &[&id_org, &id_user, &id_band])
                .await?;
        }

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
            SELECT u.id, u.pseudo, u.name, u.firstname, u.email, u.creation_stamp, u.last_login, u.verified
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

    pub async fn get_affected_users(&self, id_band: i32, id_orgs: Vec<i32>) -> Result<Vec<UserInterface>> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
            SELECT u.id, u.pseudo, u.name, u.firstname, u.email, u.creation_stamp, u.last_login, u.verified
            FROM cnm_user u JOIN org_assign oa ON oa.id_user = u.id 
            WHERE oa.id_band = $1 AND oa.id_org = $2
        ",
            )
            .await?;
        let mut res: Vec<UserInterface> = Vec::new();    
        for id_org in id_orgs {
            
            let mut loc = client
            .query(&stmt, &[&id_band, &id_org])
            .await?
            .iter()
            .map(|row| UserInterface {
                id: row.get(0),
                pseudo: row.get(1),
                name: row.get(2),
                firstname: row.get(3),
                email: row.get(4),
                creation_stamp: row.get(5),
                last_login: row.get(6),
                verified: row.get(7),
                is_admin: None,
            })
            .collect::<Vec<UserInterface>>();
            res.append(&mut loc);
        }

        Ok(res)
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
    ) -> Result<ContactInterface> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
                INSERT INTO contact(
                    id_org,
                    name,
                    firstname,
                    email,
                    phone,
                    address,
                    zip_code,
                    city,
                    id_band
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
                RETURNING 
                    id,
                    name,
                    firstname,
                    email,
                    phone,
                    address,
                    zip_code,
                    city,
                    creation_stamp
            ",
            )
            .await?;
        let rows: Vec<ContactInterface> = client
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
            .collect();
        Ok(rows[0].clone())
    }

    pub async fn update_contact(&self, contact: ContactInterface) -> Result<ContactInterface> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
            UPDATE contact
            SET 
                name = $1, 
                firstname = $2, 
                email = $3, 
                phone = $4, 
                address = $5, 
                zip_code = $6, 
                city = $7
            RETURNING 
                id,
                name,
                firstname,
                email,
                phone,
                address,
                zip_code,
                city,
                creation_stamp
        ",
            )
            .await?;
        let res: Vec<ContactInterface> = client
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
            .collect();

        Ok(res[0].clone())
    }

    pub async fn remove_contact(&self, id_contact: i32) -> Result<ContactInterface> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
                DELETE FROM contact 
                WHERE id = $1
                RETURNING 
                    id,
                    name,
                    firstname,
                    email,
                    phone,
                    address,
                    zip_code,
                    city,
                    creation_stamp
            ",
            )
            .await?;
        let res: Vec<ContactInterface> = client
            .query(&stmt, &[&id_contact])
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
            .collect();

        Ok(res[0].clone())
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
