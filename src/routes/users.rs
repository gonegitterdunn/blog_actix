use crate::errors::AppError;
use crate::routes::convert;
use crate::{models, Pool};
use actix_web::{web, HttpResponse};
use futures::Future;

#[derive(Debug, Deserialize, Serialize)]
struct UserInput {
  username: String,
}

fn create_user(
  user: web::Json<UserInput>,
  pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AppError> {
  web::block(move || {
    let username = user.into_inner().username;
    let connection = &pool.get().unwrap();
    models::create_user(connection, username.as_str())
  })
  .then(convert)
}

fn find_user(
  name: web::Path<String>,
  pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AppError> {
  web::block(move || {
    let connection = &pool.get().unwrap();
    let username = name.into_inner();
    let key = models::UserKey::Username(username.as_str());
    models::find_user(connection, key)
  })
  .then(convert)
}

fn get_user(
  user_id: web::Path<i32>,
  pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AppError> {
  web::block(move || {
    let connection = &pool.get().unwrap();
    let id = user_id.into_inner();
    let key = models::UserKey::ID(id);
    models::find_user(connection, key)
  })
  .then(convert)
}

pub fn configure(config: &mut web::ServiceConfig) {
  config
    .service(web::resource("/users").route(web::post().to_async(create_user)))
    .service(web::resource("/users/find/{name}").route(web::get().to_async(find_user)))
    .service(web::resource("/users/{id}").route(web::get().to_async(get_user)));
}
