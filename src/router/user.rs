use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

use crate::{
    auth::{create_jwt, with_jwt, Claims},
    config::Config,
    errors::Error,
    models::{band::Band, user::User},
};

#[derive(Deserialize)]
struct UserCreationRequest {
    pub pseudo: String,
    pub email: String,
    pub name: String,
    pub firstname: String,
    pub pwd: String,
}

#[derive(Serialize)]
struct UserCreationResponse {
    id: i32,
}

async fn user_create(pool: Pool, body: UserCreationRequest) -> Result<impl Reply, Rejection> {
    let user = User::new(pool);
    Ok(warp::reply::json(&UserCreationResponse {
        id: user
            .create(body.pseudo, body.email, body.name, body.firstname, body.pwd)
            .await
            .map_err(|e| Error::Database(e.to_string()))?,
    }))
}

async fn user_verify(email: String, chain: String, pool: Pool) -> Result<impl Reply, Rejection> {
    let user = User::new(pool);
    user.verify(email, chain)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(warp::reply())
}

#[derive(Deserialize)]
struct UserUpdateRequest {
    field: String,
    value: String,
}

async fn user_update(
    pool: Pool,
    claims: Claims,
    body: UserUpdateRequest,
) -> Result<impl Reply, Rejection> {
    let user = User::new(pool);
    user.update(claims.id_user, body.field, body.value)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(warp::reply())
}

async fn user_read(id: i32, pool: Pool, claims: Claims) -> Result<impl Reply, Rejection> {
    let user = User::new(pool);
    let my_bands = user
        .get_bands(claims.id_user)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    let them_bands = user
        .get_bands(id)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    if my_bands
        .iter()
        .any(|b| them_bands.iter().any(|bb| b.id == bb.id))
    {
        Ok(warp::reply::json(
            &user
                .read(id)
                .await
                .map_err(|e| Error::Database(e.to_string()))?,
        ))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

#[derive(Deserialize)]
struct UserAddBandRequest {
    email: String,
    #[serde(rename = "idBand")]
    id_band: i32,
    administrator: bool,
}

async fn user_add_band(
    pool: Pool,
    claims: Claims,
    body: UserAddBandRequest,
) -> Result<impl Reply, Rejection> {
    let user = User::new(pool.clone());
    let band = Band::new(pool);
    if let Some(uid) = user
        .get_id_from_email(body.email)
        .await
        .map_err(|e| Error::Database(e.to_string()))?
    {
        if band
            .is_admin(claims.id_user, body.id_band)
            .await
            .map_err(|e| Error::Database(e.to_string()))?
        {
            user.add_band(uid, body.id_band, body.administrator)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;
            Ok(warp::reply())
        } else {
            Err(warp::reject::custom(Error::Unauthorized))
        }
    } else {
        Err(warp::reject::custom(Error::NotFound))
    }
}

#[derive(Deserialize)]
struct ExitBandRequest {
    #[serde(rename = "idBand")]
    id_band: i32,
    pwd: String,
}

async fn user_exit_band(
    pool: Pool,
    claims: Claims,
    body: ExitBandRequest,
) -> Result<impl Reply, Rejection> {
    let user = User::new(pool);
    if user
        .authenticate_with_id(claims.id_user, body.pwd)
        .await
        .map_err(|e| Error::Database(e.to_string()))?
    {
        user.exit_band(claims.id_user, body.id_band)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(warp::reply())
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

#[derive(Deserialize)]
struct AuthenticateRequest {
    email: String,
    pwd: String,
}

#[derive(Serialize)]
struct AuthenticateResponse {
    status: bool,
    jwt: Option<String>,
}

async fn user_authenticate(pool: Pool, body: AuthenticateRequest) -> Result<impl Reply, Rejection> {
    let user = User::new(pool);

    if user
        .authenticate(body.email.clone(), body.pwd)
        .await
        .map_err(|e| Error::Database(e.to_string()))?
    {
        let id = user
            .get_id_from_email(body.email)
            .await
            .map_err(|e| Error::Database(e.to_string()))?
            .unwrap();
        let bands = user
            .get_bands(id)
            .await
            .map_err(|e| Error::Database(e.to_string()))?
            .iter()
            .map(|b| b.id)
            .collect::<Vec<i32>>();
        Ok(warp::reply::json(&AuthenticateResponse {
            status: true,
            jwt: Some(create_jwt(id, bands).map_err(|_| Error::Internal)?),
        }))
    } else {
        Ok(warp::reply::json(&AuthenticateResponse {
            status: false,
            jwt: None,
        }))
    }
}

async fn user_get_bands(pool: Pool, claims: Claims) -> Result<impl Reply, Rejection> {
    let user = User::new(pool);

    Ok(warp::reply::json(
        &user
            .get_bands(claims.id_user)
            .await
            .map_err(|e| Error::Database(e.to_string()))?,
    ))
}

pub fn user_routes(
    config: Config,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let register = warp::path!("register")
        .and(warp::post())
        .and(config.with_pool())
        .and(warp::body::json())
        .and_then(user_create);

    let verify = warp::path!("verify" / String / String)
        .and(config.with_pool())
        .and_then(user_verify);

    let update = warp::path!("update")
        .and(warp::put())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(user_update);

    let read = warp::path!("read" / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(user_read);

    let add_band = warp::path!("addband")
        .and(warp::patch())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(user_add_band);

    let exit_band = warp::path!("exitband")
        .and(warp::patch())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(user_exit_band);

    let auth = warp::path!("login")
        .and(warp::post())
        .and(config.with_pool())
        .and(warp::body::json())
        .and_then(user_authenticate);

    let bands = warp::path!("bands")
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(user_get_bands);

    register
        .or(verify)
        .or(update)
        .or(read)
        .or(add_band)
        .or(exit_band)
        .or(auth)
        .or(bands)
}
