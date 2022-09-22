use deadpool_postgres::Pool;
use serde::Deserialize;
use warp::{Filter, Rejection, Reply};

use crate::{
    auth::{with_jwt, Claims},
    config::Config,
    db_error_to_warp,
    errors::Error,
    models::{band::Band, note::Note},
};

#[derive(Deserialize)]
struct NoteCreateRequest {
    #[serde(rename = "idBand")]
    id_band: i32,
    #[serde(rename = "idActivity")]
    id_activity: i32,
    note: String,
}

async fn note_create(
    pool: Pool,
    claims: Claims,
    body: NoteCreateRequest,
) -> Result<impl Reply, Rejection> {
    let note = Note::new(pool);
    let res = note
        .create(claims.id_user, body.id_band, body.id_activity, body.note)
        .await
        .map_err(db_error_to_warp)?;
    Ok(warp::reply::json(&res))
}

#[derive(Deserialize)]
struct NoteUpdateRequest {
    id: i32,
    note: String,
}

async fn note_edit(
    pool: Pool,
    claims: Claims,
    body: NoteUpdateRequest,
) -> Result<impl Reply, Rejection> {
    let note = Note::new(pool);
    let res = note
        .edit(body.id, claims.id_user, body.note)
        .await
        .map_err(db_error_to_warp)?;
    Ok(warp::reply::json(&res))
}

async fn note_delete(
    id: i32,
    id_band: i32,
    pool: Pool,
    claims: Claims,
) -> Result<impl Reply, Rejection> {
    let band = Band::new(pool.clone());
    let note = Note::new(pool);

    if band
        .is_admin(claims.id_user, id_band)
        .await
        .map_err(db_error_to_warp)?
    {
        let res = note.delete(id).await.map_err(db_error_to_warp)?;
        Ok(warp::reply::json(&res))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

async fn note_read_all(
    id_activity: i32,
    id_band: i32,
    pool: Pool,
    _claims: Claims,
) -> Result<impl Reply, Rejection> {
    let note = Note::new(pool);
    let res = note
        .read_all(id_activity, id_band)
        .await
        .map_err(db_error_to_warp)?;
    Ok(warp::reply::json(&res))
}

pub fn note_routes(
    config: Config,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    let create = warp::path!("create")
        .and(warp::post())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(note_create);

    let edit = warp::path("edit")
        .and(warp::put())
        .and(config.with_pool())
        .and(with_jwt())
        .and(warp::body::json())
        .and_then(note_edit);

    let delete = warp::path!("delete" / i32 / i32)
        .and(warp::delete())
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(note_delete);

    let read_all = warp::path!("all" / i32 / i32)
        .and(config.with_pool())
        .and(with_jwt())
        .and_then(note_read_all);

    create.or(edit).or(delete).or(read_all)
}
