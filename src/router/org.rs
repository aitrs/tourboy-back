use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

use crate::{
    auth::{with_jwt, Claims},
    config::Config,
    errors::Error,
    models::{
        band::Band,
        filter,
        org::{ContactInterface, Org, Status, OrgRawInterface},
        user::User,
    },
    paginator::Paginator, db_error_to_warp,
};

#[derive(Serialize)]
struct ListResponse {
    orgs: Vec<OrgRawInterface>,
    pagination: Paginator,
}

pub async fn is_user_in_band(pool: Pool, claims: Claims, id_band: i32) -> anyhow::Result<bool, Error> {
    let user = User::new(pool);
    let bands = user
        .get_bands(claims.id_user)
        .await
        .map_err(db_error_to_warp)?;
    if bands.iter().any(|bi| bi.id == id_band) {
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn org_all_list(
    id_band: i32,
    page: i32,
    size: i32,
    pool: Pool,
    _claims: Claims,
    filters_str: String,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool);
    let filters: Vec<filter::FilterIntermediate> =
        serde_json::from_str(&filters_str).map_err(|_| Error::Internal)?;
    let (res, pag) = org
        .all_orgs(
            id_band,
            filters
                .iter()
                .map(|f| filter::Filter::from(f.clone()))
                .collect(),
            Some(Paginator {
                page,
                size,
                page_count: None,
                item_count: None,
            }),
        )
        .await
        .map_err(db_error_to_warp)?;
    Ok(warp::reply::json(&ListResponse {
        orgs: res,
        pagination: pag,
    }))
}

async fn org_list(
    id_band: i32,
    page: i32,
    size: i32,
    pool: Pool,
    claims: Claims,
    filters_str: String,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool);
    let filters: Vec<filter::FilterIntermediate> =
        serde_json::from_str(&filters_str).map_err(|_| Error::Internal)?;
    let (res, pag) = org
        .band_related_orgs_and_statuses(
            claims.id_user,
            id_band,
            filters
                .iter()
                .map(|f| filter::Filter::from(f.clone()))
                .collect(),
            Some(Paginator {
                page,
                size,
                page_count: None,
                item_count: None,
            }),
        )
        .await
        .map_err(db_error_to_warp)?;
    Ok(warp::reply::json(&ListResponse {
        orgs: res,
        pagination: pag,
    }))
}

#[derive(Deserialize)]
struct TagRequest {
    status: String,
    orgs: Vec<i32>,
}

#[derive(Serialize)]
struct TagResponse {
    tagged: bool,
    reason: Option<String>,
}

async fn org_tag(
    id_band: i32,
    id_user: i32,
    pool: Pool,
    claims: Claims,
    body: TagRequest,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool.clone());
    let band = Band::new(pool);
    let users = band
        .get_band_members(id_band)
        .await
        .map_err(db_error_to_warp)?;
    let is_admin = band
        .is_admin(claims.id_user, id_band)
        .await
        .map_err(db_error_to_warp)?;
    let assigned = org
        .get_affected_users(id_band, body.orgs.clone())
        .await
        .map_err(db_error_to_warp)?;
    let is_assigned = assigned.iter().any(|u| u.id == id_user);

    if users.iter().any(|u| u.id == id_user) && (is_admin || is_assigned) {
        org.tag_orgs(id_user, id_band, body.orgs, Status::from(body.status))
            .await
            .map_err(db_error_to_warp)?;

        Ok(warp::reply::json(&TagResponse {
            tagged: true,
            reason: None,
        }))
    } else {
        Ok(warp::reply::json(&TagResponse {
            tagged: false,
            reason: Some(if is_admin {
                "Cet utilisateur ne fait pas partie du groupe courant".to_string()
            } else {
                "Droits insuffisants".to_string()
            }),
        }))
    }
}

async fn org_categories(pool: Pool, _: Claims) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool);
    Ok(warp::reply::json(
        &org.get_categories()
            .await
            .map_err(db_error_to_warp)?,
    ))
}

async fn org_assigned_users(
    id_band: i32,
    pool: Pool,
    claims: Claims,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool.clone());

    if is_user_in_band(pool, claims, id_band).await? {
        Ok(warp::reply::json(
            &org.get_assigned_users(id_band)
                .await
                .map_err(db_error_to_warp)?,
        ))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

async fn org_contacts(
    id_org: i32,
    id_band: i32,
    pool: Pool,
    claims: Claims,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool.clone());

    if is_user_in_band(pool, claims, id_band).await? {
        Ok(warp::reply::json(
            &org.get_contacts(id_org, id_band)
                .await
                .map_err(db_error_to_warp)?,
        ))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

async fn org_create_contact(
    id_org: i32,
    id_band: i32,
    pool: Pool,
    claims: Claims,
    body: ContactInterface,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool.clone());

    if is_user_in_band(pool, claims, id_band).await? {
        let res = org
            .add_contact(id_org, id_band, body)
            .await
            .map_err(db_error_to_warp)?;
        Ok(warp::reply::json(&res))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

async fn org_update_contact(
    pool: Pool,
    claims: Claims,
    body: ContactInterface,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool.clone());
    let id_band = org
        .get_contact_band_id(body.id)
        .await
        .map_err(db_error_to_warp)?;

    if is_user_in_band(pool, claims, id_band).await? {
        org.update_contact(body)
            .await
            .map_err(db_error_to_warp)?;
        Ok(warp::reply())
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

async fn org_delete_contact(
    id_contact: i32,
    pool: Pool,
    claims: Claims,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool.clone());
    let id_band = org
        .get_contact_band_id(id_contact)
        .await
        .map_err(db_error_to_warp)?;

    if is_user_in_band(pool, claims, id_band).await? {
        org.remove_contact(id_contact)
            .await
            .map_err(db_error_to_warp)?;
        Ok(warp::reply())
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

pub fn org_routes(
    config: Config,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let list_route = warp::path!("list" / i32 / i32 / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::header("filters"))
        .and_then(org_list);

    let all_route = warp::path!("all" / i32 / i32 / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::header("filters"))
        .and_then(org_all_list);

    let tag_route = warp::path!("tag" / i32 / i32)
        .and(warp::patch())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(org_tag);

    let cat_route = warp::path("categories")
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(org_categories);

    let assigned_route = warp::path!("assigned" / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(org_assigned_users);

    let get_contacts_route = warp::path!("contact" / i32 / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(org_contacts);

    let create_contact_route = warp::path!("contact" / i32 / i32)
        .and(warp::post())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(org_create_contact);

    let update_contact_route = warp::path!("contact")
        .and(warp::put())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(org_update_contact);

    let delete_contact_route = warp::path!("contact" / i32)
        .and(warp::delete())
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(org_delete_contact);

    list_route
        .or(all_route)
        .or(tag_route)
        .or(cat_route)
        .or(assigned_route)
        .or(get_contacts_route)
        .or(create_contact_route)
        .or(update_contact_route)
        .or(delete_contact_route)
}
