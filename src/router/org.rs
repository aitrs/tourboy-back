use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

use crate::{
    auth::{with_jwt, Claims},
    config::Config,
    errors::Error,
    models::{
        filter,
        org::{ContactInterface, Org, OrgInterface, Status},
        user::User,
    },
    paginator::Paginator,
};

#[derive(Serialize)]
struct ListResponse {
    orgs: Vec<OrgInterface>,
    pagination: Paginator,
}

pub async fn is_user_in_band(pool: Pool, claims: Claims, id_band: i32) -> Result<bool, Error> {
    let user = User::new(pool);
    let bands = user
        .get_bands(claims.id_user)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    if bands.iter().any(|bi| bi.id == id_band) {
        Ok(true)
    } else {
        Ok(false)
    }
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
    let filters: Vec<filter::Filter> =
        serde_json::from_str(&filters_str).map_err(|_| Error::Internal)?;
    let (res, pag) = org
        .band_related_orgs_and_statuses(
            claims.id_user,
            id_band,
            filters,
            Some(Paginator {
                page,
                size,
                page_count: None,
                item_count: None,
            }),
        )
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(warp::reply::json(&ListResponse {
        orgs: res,
        pagination: pag,
    }))
}

#[derive(Deserialize)]
struct TagRequest {
    status: Status,
}

async fn org_tag(
    id_band: i32,
    id_org: i32,
    pool: Pool,
    claims: Claims,
    body: TagRequest,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool);
    org.tag_org(claims.id_user, id_band, id_org, body.status)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(warp::reply())
}

async fn org_categories(pool: Pool, _: Claims) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool);
    Ok(warp::reply::json(
        &org.get_categories()
            .await
            .map_err(|e| Error::Database(e.to_string()))?,
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
                .map_err(|e| Error::Database(e.to_string()))?,
        ))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

#[derive(Deserialize)]
struct ContactRequest {
    contact: ContactInterface,
}

#[derive(Serialize)]
struct ContactCreationResponse {
    id: i32,
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
                .map_err(|e| Error::Database(e.to_string()))?,
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
    body: ContactRequest,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool.clone());

    if is_user_in_band(pool, claims, id_band).await? {
        let res = org
            .add_contact(id_org, id_band, body.contact)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(warp::reply::json(&ContactCreationResponse { id: res }))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

async fn org_update_contact(
    pool: Pool,
    claims: Claims,
    body: ContactRequest,
) -> Result<impl Reply, Rejection> {
    let org = Org::new(pool.clone());
    let id_band = org
        .get_contact_band_id(body.contact.id)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    if is_user_in_band(pool, claims, id_band).await? {
        org.update_contact(body.contact)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
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
        .map_err(|e| Error::Database(e.to_string()))?;

    if is_user_in_band(pool, claims, id_band).await? {
        org.remove_contact(id_contact)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
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

    let get_contacts_route = warp::path!("contacts" / i32 / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(org_contacts);

    let create_contact_route = warp::path!("contacts" / i32 / i32)
        .and(warp::post())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(org_create_contact);

    let update_contact_route = warp::path!("contacts")
        .and(warp::put())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(org_update_contact);

    let delete_contact_route = warp::path!("contacts" / i32)
        .and(warp::delete())
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(org_delete_contact);

    list_route
        .or(tag_route)
        .or(cat_route)
        .or(assigned_route)
        .or(get_contacts_route)
        .or(create_contact_route)
        .or(update_contact_route)
        .or(delete_contact_route)
}
