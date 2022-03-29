use crate::errors::AppError;
use actix_web::HttpResponse;

pub(super) mod posts;
pub(super) mod users;
pub(super) mod comments;

fn convert<T, E>(result: Result<T, E>) -> Result<HttpResponse, AppError>
where
  T: serde::Serialize,
  AppError: From<E>,
{
  result
    .map(|res| HttpResponse::Ok().json(res))
    .map_err(Into::into)
}
