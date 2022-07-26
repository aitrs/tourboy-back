use std::{env, fs};

use anyhow::Result;
use deadpool_postgres::{ManagerConfig, Pool, RecyclingMethod, Runtime};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use warp::{Filter, Rejection};

use crate::errors::Error;

const DEFAULT_CONF_FILE: &str = "/ect/cnm/cnm.json";
const ENV_CONF_KEY: &str = "CNM_CONFIG";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    host: String,
    user: String,
    name: String,
    password: Option<String>,
    port: Option<u16>,
}

impl Database {
    pub fn host(&self) -> String {
        self.host.clone()
    }
    pub fn user(&self) -> String {
        self.user.clone()
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn password(&self) -> Option<String> {
        self.password.clone()
    }
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn to_postgres_url(&self) -> tokio_postgres::Config {
        let mut config = tokio_postgres::Config::new();
        config.host(&self.host);
        config.user(&self.user);
        config.dbname(&self.name);
        if let Some(password) = self.password.clone() {
            config.password(password);
        }

        if let Some(port) = self.port {
            config.port(port);
        }

        config
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    database: Database,
    #[serde(skip)]
    pool: Option<Pool>,
}

async fn check_pool(pool: Option<Pool>) -> std::result::Result<Pool, Rejection> {
    if let Some(p) = pool {
        Ok(p)
    } else {
        Err(warp::reject::custom(Error::UnableToGetDatabasePool))
    }
}

impl Config {
    pub fn retrieve() -> Result<Self> {
        let path = if let Some((_, v)) = env::vars().find(|(key, _)| key == ENV_CONF_KEY) {
            v
        } else {
            DEFAULT_CONF_FILE.to_string()
        };
        let fcontents = fs::read_to_string(path)?;
        let mut config: Config = serde_json::from_str(&fcontents)?;

        let mut dpconf = deadpool_postgres::Config::new();
        dpconf.dbname = Some(config.database().name());
        dpconf.host = Some(config.database().host());
        dpconf.user = Some(config.database().user());
        dpconf.port = config.database().port();
        dpconf.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        let pool = dpconf.create_pool(Some(Runtime::Tokio1), NoTls)?;

        config.set_pool(pool);

        Ok(config)
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    fn set_pool(&mut self, pool: Pool) {
        self.pool = Some(pool);
    }

    pub fn with_pool(&self) -> impl Filter<Extract = (Pool,), Error = Rejection> + Clone {
        let p = self.pool.clone();
        warp::any().map(move || p.clone()).and_then(check_pool)
    }
}
