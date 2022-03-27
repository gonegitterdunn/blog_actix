use actix_web::error::BlockingError;
use actix_web::HttpResponse;
use diesel::result::{
  DatabaseErrorKind::UniqueViolation,
  Error::{DatabaseError, NotFound},
};
use std::fmt::{Display, Formatter, Result};

// 1 - make enum
// 2 - impl Display

#[derive(Debug)]
pub enum AppError {
  RecordAlreadyExists,
  RecordNotFound,
  DatabaseError(diesel::result::Error),
  OperationCanceled,
}

impl Display for AppError {
  fn fmt(&self, f: &mut Formatter) -> Result {
    match self {
      AppError::RecordAlreadyExists => write!(f, "Record violates unique constraint"),
      AppError::RecordNotFound => write!(f, "Record does not exist"),
      AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
      AppError::OperationCanceled => write!(f, "Operation was canceled"),
    }
  }
}

impl From<diesel::result::Error> for AppError {
  fn from(err: diesel::result::Error) -> Self {
    match err {
      DatabaseError(UniqueViolation, _) => AppError::RecordAlreadyExists,
      NotFound => AppError::RecordNotFound,
      _ => AppError::DatabaseError(err),
    }
  }
}

impl From<BlockingError<AppError>> for AppError {
  fn from(err: BlockingError<AppError>) -> Self {
    match err {
      BlockingError::Error(inner) => inner,
      BlockingError::Canceled => AppError::OperationCanceled,
    }
  }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
  err: String,
}

impl actix_web::ResponseError for AppError {
  fn error_response(&self) -> HttpResponse {
    let err = format!("{}", self);
    let mut builder = match self {
      AppError::RecordAlreadyExists => HttpResponse::BadRequest(),
      AppError::RecordNotFound => HttpResponse::NotFound(),
      _ => HttpResponse::InternalServerError(),
    };
    builder.json(ErrorResponse { err })
  }

  fn render_response(&self) -> HttpResponse {
    self.error_response()
  }
}
