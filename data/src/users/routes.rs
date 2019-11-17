use actix_web::{error::BlockingError, web, Error, HttpResponse};
use diesel::result::{DatabaseErrorKind::UniqueViolation, Error::DatabaseError};
use futures::Future;
use serde::Deserialize;

use crate::users::models::{AuthenticationError, User};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    name: String,
    email: String,
    password: String,
}

pub fn create(
    body: web::Json<CreateUserRequest>,
    pool: web::Data<crate::DbPool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let conn = pool
        .get_ref()
        .get()
        .expect("Could not establish a database connection");
    web::block(move || User::register(&body.name, &body.email, &body.password, &conn)).then(|res| {
        match res {
            Err(BlockingError::Error(err)) => match err {
                AuthenticationError::DatabaseError(e) => match e {
                    DatabaseError(UniqueViolation, _) => Ok(HttpResponse::Conflict().into()),
                    _ => Ok(HttpResponse::InternalServerError().into()),
                },
                _ => Ok(HttpResponse::InternalServerError().into()),
            },
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
            Ok(user) => Ok(HttpResponse::Created().json(user)),
        }
    })
}

pub fn build_routes() -> actix_web::Resource {
    web::resource("").route(web::post().to_async(create))
}
