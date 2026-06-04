//! HTTP API for cell grid and entities.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::db::{fetch_all_cells, fetch_cells_on_deck, fetch_entities, CellApiRow, GridEntity};
use crate::AppState;

#[derive(Serialize)]
struct CellsResponse {
    deck: i32,
    cells: Vec<CellApiRow>,
}

#[derive(Serialize)]
struct EntitiesResponse {
    entities: Vec<GridEntity>,
}

#[derive(Serialize)]
struct AllCellsResponse {
    cells: Vec<CellApiRow>,
}

pub async fn all_cells(State(state): State<AppState>) -> impl IntoResponse {
    let Some(pool) = &state.db else {
        return (StatusCode::SERVICE_UNAVAILABLE, "database not configured").into_response();
    };
    match fetch_all_cells(pool).await {
        Ok(cells) => Json(AllCellsResponse { cells }).into_response(),
        Err(e) => {
            tracing::error!("fetch_all_cells: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn deck_cells(
    State(state): State<AppState>,
    Path(deck_z): Path<i32>,
) -> impl IntoResponse {
    let Some(pool) = &state.db else {
        return (StatusCode::SERVICE_UNAVAILABLE, "database not configured").into_response();
    };
    if !(0..=19).contains(&deck_z) {
        return (StatusCode::BAD_REQUEST, "deck must be 0..19").into_response();
    }
    match fetch_cells_on_deck(pool, deck_z).await {
        Ok(cells) => Json(CellsResponse {
            deck: deck_z,
            cells,
        })
        .into_response(),
        Err(e) => {
            tracing::error!("fetch_cells_on_deck: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn entities(State(state): State<AppState>) -> impl IntoResponse {
    let Some(pool) = &state.db else {
        return (StatusCode::SERVICE_UNAVAILABLE, "database not configured").into_response();
    };
    match fetch_entities(pool).await {
        Ok(entities) => Json(EntitiesResponse { entities }).into_response(),
        Err(e) => {
            tracing::error!("fetch_entities: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
