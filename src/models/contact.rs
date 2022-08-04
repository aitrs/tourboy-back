use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use super::{org::OrgInterface, band::BandInterface};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInterface {
    pub id: i32,
    pub org: Option<OrgInterface>,
    pub band: Option<BandInterface>,
    pub name: String,
    pub firstname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    #[serde(rename = "zipCode")]
    pub zip_code: Option<String>,
    pub city: Option<String>,
    #[serde(rename = "creationStamp")]
    pub creation_stamp: NaiveDateTime,
}

pub struct Contact(Pool);

impl Contact {
    pub fn new(pool: Pool) -> Self {
        Contact(pool)
    }

    async fn create(&self, id_org: i32, id_band: i32, body: ContactInterface) -> Result<ContactInterface> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached(
                "
                    INSERT INTO
                        contact(
                            id_org, name, firstname,
                            email, phone, address, zip_code,
                            city, id_band
                        )
                    VALUES (
                        $1, $2, $3, 
                        $4, $5, $6, $7,
                        $8, $9
                    )

                    RETURNING
                        id, name, firstname, email,
                        phone, address, zip_code, city,
                        creation_stamp,
                "
            )
            .await?;
        let rows = client
            .query(&stmt, &[
                &id_org,
                &body.name,
                &body.firstname,
                &body.email,
                &body.phone,
                &body.zip_code,
                &body.city,
                &id_band,
            ])
            .await?
            .iter()
            .map(|row| ContactInterface {
                id: row.get(0),
                name: row.get(1),
                firstname: row.get(2),
                email: row.get(3),
                phone: row.get(4),
                address: row.get(5),
                zip_code: row.get(6),
                city: row.get(7),
                creation_stamp: row.get(8),
                org: None,
                band: None,
            })
            .collect::<Vec<ContactInterface>>();

        Ok(rows[0].clone())
    }

    async fn edit(&self, id: i32, contact: ContactInterface) -> Result<ContactInterface> {
        let client = self.0.get().await?;
        let stmt = client
            .prepare_cached("
                UPDATE contact SET
                    name = $1,
                    firstname = $2,
                    email = $3,
                    phone = $4,
                    address = $5,
                    zip_code= $6,
                    city = $7,
                WHERE id = $8,
                RETURNING
                    id, name, firstname, email,
                    phone, address, zip_code, city,
                    creation_stamp,
            ")
            .await?;
        let rows = client
            .query(&stmt, &[
                &contact.name,
                &contact.firstname,
                &contact.email,
                &contact.phone,
                &contact.address,
                &contact.zip_code,
                &contact.city,
                &id,
            ])
            .await?
            .iter()
            .map(|row| ContactInterface {
                id: row.get(0),
                name: row.get(1),
                firstname: row.get(2),
                email: row.get(3),
                phone: row.get(4),
                address: row.get(5),
                zip_code: row.get(6),
                city: row.get(7),
                creation_stamp: row.get(8),
                org: None,
                band: None,
            })
            .collect::<Vec<ContactInterface>>();
        
        Ok(rows[0].clone())
    }
}