use crate::errors::AppError;
use crate::schema::comments;
use crate::schema::posts;
use crate::schema::users;
use diesel::prelude::*;

type Result<T> = std::result::Result<T, AppError>;

#[derive(Queryable, Identifiable, Serialize, Debug, PartialEq)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Queryable, Associations, Identifiable, Serialize, Debug)]
#[belongs_to(User)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Queryable, Identifiable, Associations, Serialize, Debug)]
#[belongs_to(User)]
#[belongs_to(Post)]
pub struct Comment {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub body: String,
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

pub fn publish_post(conn: &SqliteConnection, post_id: i32) -> Result<Post> {
    conn.transaction(|| {
        diesel::update(posts::table.filter(posts::id.eq(post_id)))
            .set(posts::published.eq(true))
            .execute(conn)?;

        posts::table
            .find(post_id)
            .select(posts::all_columns)
            .first(conn)
            .map_err(Into::into)
    })
}

pub enum UserKey<'a> {
    Username(&'a str),
    ID(i32),
}

pub fn find_user<'a>(conn: &SqliteConnection, key: UserKey<'a>) -> Result<User> {
    match key {
        UserKey::Username(name) => users::table
            .filter(users::username.eq(name))
            .select((users::id, users::username))
            .first::<User>(conn)
            .map_err(Into::into),
        UserKey::ID(id) => users::table
            .find(id)
            .select((users::id, users::username))
            .first::<User>(conn)
            .map_err(Into::into),
    }
}

pub fn all_posts(conn: &SqliteConnection) -> Result<Vec<(Post, User)>> {
    posts::table
        .order(posts::id.desc())
        .filter(posts::published.eq(true))
        .inner_join(users::table)
        .select((posts::all_columns, (users::id, users::username)))
        .load::<(Post, User)>(conn)
        .map_err(Into::into)
}

pub fn user_posts(
    conn: &SqliteConnection,
    user_id: i32,
) -> Result<Vec<(Post, Vec<(Comment, User)>)>> {
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
