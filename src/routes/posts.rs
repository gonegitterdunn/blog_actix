use crate::errors::AppError;
use crate::routes::convert;
use crate::{models, Pool};
use actix_web::web::{Data, Json, Path};
use actix_web::*;
use diesel::prelude::*;
use futures::Future;

#[derive(Debug, Serialize, Deserialize)]
struct PostInput {
  title: String,
  body: String,
}

fn add_post(
  user_id: Path<i32>,
  post: Json<PostInput>,
  pool: Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AppError> {
  web::block(move || {
    let conn: &SqliteConnection = &pool.get().unwrap();
    let key = models::UserKey::ID(user_id.into_inner());
    models::find_user(conn, key).and_then(|user| {
      let post = post.into_inner();
      let title = post.title;
      let body = post.body;
      models::create_post(conn, &user, title.as_str(), body.as_str())
    })
  })
  .then(convert)
}

fn publish_post(
  post_id: Path<i32>,
  pool: Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AppError> {
  web::block(move || {
    let conn: &SqliteConnection = &pool.get().unwrap();
    models::publish_post(conn, post_id.into_inner())
  })
  .then(convert)
}

fn user_posts(
  user_id: Path<i32>,
  pool: Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = AppError> {
  web::block(move || {
    let conn: &SqliteConnection = &pool.get().unwrap();
    models::fetch_user_posts(conn, user_id.into_inner())
  })
  .then(convert)
}

fn all_posts(pool: Data<Pool>) -> impl Future<Item = HttpResponse, Error = AppError> {
  web::block(move || {
    let conn: &SqliteConnection = &pool.get().unwrap();
    models::fetch_all_posts(conn)
  })
  .then(convert)
}

pub fn configure(config: &mut web::ServiceConfig) {
  config
    .service(
      web::resource("/users/{id}/posts")
        .route(web::post().to_async(add_post))
        .route(web::get().to_async(user_posts)),
    )
    .service(web::resource("/posts").route(web::get().to_async(all_posts)))
    .service(web::resource("/posts/{id}/publish").route(web::post().to_async(publish_post)));
}
