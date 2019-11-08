use actix_web::{web, Error, HttpResponse};
use futures::Future;
use serde::Deserialize;

use crate::users::models::User;

#[derive(Deserialize)]
pub struct UserCreateIO {
    name: String,
    email: String,
    password: String,
}

pub fn create(
    body: web::Json<UserCreateIO>,
    pool: web::Data<crate::DbPool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let conn = pool
        .get_ref()
        .get()
        .expect("Could not establish a database connection");
    web::block(move || User::register(&body.name, &body.email, &body.password, &conn)).then(|res| {
        match res {
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
            Ok(user) => Ok(HttpResponse::Created().json(user)),
        }
    })
}

pub fn build_routes() -> actix_web::Resource {
    web::resource("").route(web::post().to_async(create))
}
