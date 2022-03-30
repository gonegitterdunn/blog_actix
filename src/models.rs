use crate::errors::AppError;
use crate::schema::{comments, posts, users};
use diesel::prelude::*;

type Result<T> = std::result::Result<T, AppError>;

type PostWithAuthor = (Post, User);
type CommentWithUser = (Comment, User);
type CommentsWithUsers = Vec<CommentWithUser>;

// type Result<UserPosts> = Result<Vec<(Post, CommentsWithUsers)>>;

#[derive(Queryable, Identifiable, Serialize, Debug, PartialEq)]
pub struct User {
  pub id: i32,
  pub username: String,
}

#[derive(Debug, Serialize, Queryable, Identifiable, Associations)]
#[belongs_to(User)]
pub struct Post {
  pub id: i32,
  pub user_id: i32,
  pub title: String,
  pub body: String,
  pub published: bool,
}

#[derive(Debug, Serialize, Identifiable, Queryable, Associations)]
#[belongs_to(User)]
#[belongs_to(Post)]
pub struct Comment {
  pub id: i32,
  pub user_id: i32,
  pub post_id: i32,
  pub body: String,
}

#[derive(Debug, Serialize, Queryable)]
pub struct PostWithComment {
  pub id: i32,
  pub title: String,
  pub published: bool,
}

pub fn create_user(conn: &SqliteConnection, username: &str) -> Result<User> {
  conn.transaction(|| {
    diesel::insert_into(users::table)
      .values((users::username.eq(username),))
      .execute(conn)?;

    users::table
      .order(users::id.desc())
      .select((users::id, users::username))
      .first(conn)
      .map_err(Into::into)
  })
}

pub enum UserKey<'a> {
  Username(&'a str),
  ID(i32),
}

pub fn find_user(conn: &SqliteConnection, key: UserKey) -> Result<User> {
  match key {
    UserKey::Username(name) => users::table
      .filter(users::username.eq(name))
      .select((users::id, users::username))
      .first::<User>(conn)
      .map_err(AppError::from),
    UserKey::ID(id) => users::table
      .find(id)
      .select((users::id, users::username))
      .first::<User>(conn)
      .map_err(AppError::from),
  }
}

pub fn create_post(conn: &SqliteConnection, user: &User, title: &str, body: &str) -> Result<Post> {
  conn.transaction(|| {
    diesel::insert_into(posts::table)
      .values((
        posts::user_id.eq(user.id),
        posts::title.eq(title),
        posts::body.eq(body),
      ))
      .execute(conn)?;

    posts::table
      .order(posts::id.desc())
      .select(posts::all_columns)
      .first(conn)
      .map_err(Into::into)
  })
}

pub fn publish_post(conn: &SqliteConnection, user_id: i32) -> Result<Post> {
  conn.transaction(|| {
    diesel::update(posts::table.filter(posts::id.eq(user_id)))
      .set(posts::published.eq(true))
      .execute(conn)?;

    posts::table
      .find(posts::id)
      .select(posts::all_columns)
      .first(conn)
      .map_err(Into::into)
  })
}

pub fn fetch_all_posts(
  conn: &SqliteConnection,
) -> Result<Vec<(PostWithAuthor, CommentsWithUsers)>> {
  let query = posts::table
    .order(posts::id.desc())
    .filter(posts::published.eq(true))
    .inner_join(users::table)
    .select((posts::all_columns, (users::id, users::username)));

  // vector of tuples
  let post_with_users = query.load::<(Post, User)>(conn)?;
  // tuple of vectors
  let (posts, post_users): (Vec<_>, Vec<_>) = post_with_users.into_iter().unzip();

  let comments = Comment::belonging_to(&posts)
    .inner_join(users::table)
    .select((comments::all_columns, (users::id, users::username)))
    .load::<(Comment, User)>(conn)?
    .grouped_by(&posts); // associates comments indexed by posts -- Vector wrapper

  Ok(posts.into_iter().zip(post_users).zip(comments).collect())
}

pub fn fetch_user_posts(
  conn: &SqliteConnection,
  user_id: i32,
) -> Result<Vec<(Post, Vec<CommentWithUser>)>> {
  let posts = posts::table
    .filter(posts::user_id.eq(user_id))
    .order(posts::id.desc())
    .select(posts::all_columns)
    .load::<Post>(conn)?;

  let comments = Comment::belonging_to(&posts)
    .inner_join(users::table)
    .select((comments::all_columns, (users::id, users::username)))
    .load::<(Comment, User)>(conn)?
    .grouped_by(&posts);

  Ok(posts.into_iter().zip(comments).collect())
}

pub fn create_comment(
  conn: &SqliteConnection,
  user_id: i32,
  post_id: i32,
  body: &str,
) -> Result<Comment> {
  conn.transaction(|| {
    diesel::insert_into(comments::table)
      .values((
        comments::user_id.eq(user_id),
        comments::post_id.eq(post_id),
        comments::body.eq(body),
      ))
      .execute(conn)?;

    comments::table
      .order(comments::id.desc())
      .select(comments::all_columns)
      .first(conn)
      .map_err(Into::into)
  })
}

pub fn fetch_post_comments(conn: &SqliteConnection, post_id: i32) -> Result<Vec<(Comment, User)>> {
  comments::table
    .filter(comments::post_id.eq(post_id))
    .inner_join(users::table)
    .select((comments::all_columns, (users::id, users::username)))
    .load::<(Comment, User)>(conn)
    .map_err(Into::into)
}

pub fn fetch_user_comments(
  conn: &SqliteConnection,
  user_id: i32,
) -> Result<Vec<(Comment, PostWithComment)>> {
  comments::table
    .filter(comments::user_id.eq(user_id))
    .inner_join(posts::table)
    .select((
      comments::all_columns,
      (posts::id, posts::title, posts::published),
    ))
    .load::<(Comment, PostWithComment)>(conn)
    .map_err(Into::into)
}
