use crate::db::DbPool;
use axum::extract::State;
use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct HelloResponse<'a> {
    message: &'a str,
}

pub async fn hello<'a>(_: State<DbPool>) -> (StatusCode, Json<HelloResponse<'a>>) {
    (
        StatusCode::CREATED,
        Json(HelloResponse {
            message: "hello, wonderful",
        }),
    )
}
