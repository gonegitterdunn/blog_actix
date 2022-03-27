use crate::errors::AppError;
use actix_web::HttpResponse;

pub(super) mod users;

fn convert<T, E>(result: Result<T, E>) -> Result<HttpResponse, AppError>
where
  T: serde::Serialize,
  AppError: From<E>,
{
  result
    .map(|res| HttpResponse::Ok().json(res))
    .map_err(Into::into)
}
