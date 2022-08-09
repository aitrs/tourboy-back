use std::fs;

use anyhow::{anyhow, Result};
use deadpool_postgres::Pool;
use serde::Serialize;
use tinytemplate::TinyTemplate;
use lettre::{message::{MessageBuilder, Mailbox, MultiPart}, Address, Transport};
use crate::config::Config;

#[derive(Serialize, Debug)]
struct VerifContext {
    link: String,
    pseudo: String,
    mail: String,
}
#[derive(Debug)]
pub enum Mailer {
    Verify,
    ForgotPassword,
}

impl Mailer {
    pub async fn send_email(&self, user_id: i32, pool: Pool) -> Result<()> {
        let config = Config::retrieve(false)?;
        let client = pool.get().await?;
        let stmt = client.prepare("
            SELECT verify_chain, pseudo, email FROM cnm_user
            WHERE id = $1
        ").await?;
        let rows = client.query(&stmt, &[&user_id])
            .await?
            .iter()
            .map(|row| {
                let v: String = row.get(0);
                let p: String = row.get(1);
                let e: String = row.get(2);
                (v, p, e)
            })
            .collect::<Vec<(String, String, String)>>();
        
        if rows.is_empty() {
            Err(anyhow!("No use found"))
        } else {
            let rawcontents = fs::read_to_string(
                match self {
                    Self::ForgotPassword => config.forgot_password_mail(),
                    Self::Verify => config.verif_mail(), 
                }
            )?;
            let context = VerifContext {
                link: format!(
                    "{}/{}/{}", 
                    match self {
                        Self::Verify => config.verif_base_url(),
                        Self::ForgotPassword => config.forgot_password_base_url(),
                    },
                    user_id,
                    rows[0].0,
                ),
                pseudo: rows[0].1.clone(),
                mail: config.admin_mail(),
            };
            let mut tt = TinyTemplate::new();
                
            tt.add_template("verifmail", &rawcontents)?;
            let mail_contents = tt.render("verifmail", &context)?;
            let email = MessageBuilder::new()
                .from(Mailbox::new(Some("Noreply Tourboy".to_string()), config.from_addr().parse::<Address>()?))
                .to(Mailbox::new(None, rows[0].2.parse::<Address>()?))
                .subject("Vérifiez votre email sur Tourboy")
                .multipart(
                    MultiPart::alternative_plain_html(
                        String::from("
                            Vous avez besoin d'un affichage HTML
                            pour visionner ce message correctement.
                            Veuillez contacter dorian.vuolo@gmail.com
                            pour une intervention manuelle.
                        "), 
                        mail_contents,
                    )
                )?;

            config
                .mailer()?
                .send(&email)?;
                
            Ok(())
        }   
    }
}