
use crate::errors:AppError;
use actix_web::HttpResponse;

pub(super) mod users;

fn convert<T, E>(result: Result<T, E>) -> Result<HttpResponse, AppError> where
  T: serde::Serialize,
  AppError: From<E>
  {
    result.map(|d| HttpResponse::Ok().json(d)).map_err(Into::into)
  }