use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

use crate::{
    auth::{with_jwt, Claims},
    config::Config,
    db_error_to_warp,
    errors::Error,
    models::{band::Band, user::UserInterface},
};

use super::org::is_user_in_band;

#[derive(Deserialize)]
struct BandCreateRequest {
    pub name: String,
}

#[derive(Serialize)]
struct BandCreateResponse {
    pub id: i32,
}

async fn band_create(
    pool: Pool,
    claims: Claims,
    body: BandCreateRequest,
) -> Result<impl Reply, Rejection> {
    let band = Band::new(pool);
    let id = band
        .create(claims.id_user, body.name)
        .await
        .map_err(db_error_to_warp)?;
    Ok(warp::reply::json(&BandCreateResponse { id }))
}

async fn band_remove(id_band: i32, pool: Pool, claims: Claims) -> Result<impl Reply, Rejection> {
    let band = Band::new(pool);
    if band
        .is_admin(claims.id_user, id_band)
        .await
        .map_err(db_error_to_warp)?
    {
        band.remove(id_band).await.map_err(|_| Error::Internal)?;
        Ok(warp::reply())
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

#[derive(Serialize)]
struct BandIsAdminResponse {
    #[serde(rename = "isAdmin")]
    is_admin: bool,
}

async fn band_is_admin(id_band: i32, pool: Pool, claims: Claims) -> Result<impl Reply, Rejection> {
    let band = Band::new(pool);
    Ok(warp::reply::json(&BandIsAdminResponse {
        is_admin: band
            .is_admin(claims.id_user, id_band)
            .await
            .map_err(db_error_to_warp)?,
    }))
}

async fn get_band_admins(
    id_band: i32,
    pool: Pool,
    claims: Claims,
) -> Result<impl Reply, Rejection> {
    let band = Band::new(pool.clone());
    if is_user_in_band(pool, claims, id_band)
        .await
        .map_err(|e| Error::Database(e.to_string()))?
    {
        Ok(warp::reply::json(
            &band
                .get_band_members(id_band)
                .await
                .map_err(db_error_to_warp)?
                .iter()
                .filter(|user| user.is_admin.unwrap_or(false))
                .cloned()
                .collect::<Vec<UserInterface>>(),
        ))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

async fn band_edit(
    id_band: i32,
    pool: Pool,
    claims: Claims,
    body: BandCreateRequest,
) -> Result<impl Reply, Rejection> {
    let band = Band::new(pool);
    if band
        .is_admin(claims.id_user, id_band)
        .await
        .map_err(db_error_to_warp)?
    {
        band.edit(id_band, body.name)
            .await
            .map_err(db_error_to_warp)?;
        Ok(warp::reply())
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

#[derive(Serialize)]
struct BandAdminCountResponse {
    count: i32,
}

async fn band_admin_count(id_band: i32, pool: Pool) -> Result<impl Reply, Rejection> {
    let band = Band::new(pool);
    Ok(warp::reply::json(&BandAdminCountResponse {
        count: band
            .get_admin_count(id_band)
            .await
            .map_err(db_error_to_warp)?,
    }))
}

#[derive(Serialize)]
struct BandMembersResponse {
    members: Vec<UserInterface>,
}

async fn band_members(id_band: i32, pool: Pool, _claims: Claims) -> Result<impl Reply, Rejection> {
    let band = Band::new(pool);

    Ok(warp::reply::json(&BandMembersResponse {
        members: band
            .get_band_members(id_band)
            .await
            .map_err(db_error_to_warp)?,
    }))
}

pub fn band_routes(
    config: Config,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    let create_route = warp::path!("add")
        .and(warp::post())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(band_create);

    let remove_route = warp::path!("del" / i32)
        .and(warp::delete())
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(band_remove);

    let update_route = warp::path!("upd" / i32)
        .and(warp::put())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(band_edit);

    let ba_count_route = warp::path!("admcount" / i32)
        .and(config.with_pool())
        .and_then(band_admin_count);

    let members_route = warp::path!("members" / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(band_members);

    let is_admin_route = warp::path!("isadmin" / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(band_is_admin);

    let admins_route = warp::path!("admins" / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(get_band_admins);

    create_route
        .or(remove_route)
        .or(update_route)
        .or(ba_count_route)
        .or(members_route)
        .or(is_admin_route)
        .or(admins_route)
}
