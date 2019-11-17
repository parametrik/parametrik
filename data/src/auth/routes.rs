use actix_web::{web, Error, HttpResponse};
use futures::Future;
use serde::{Deserialize, Serialize};

use crate::auth::token::create_token;
use crate::users::models::User;
use crate::{DbPool, Keypair};

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
}

pub fn login(
    body: web::Json<LoginRequest>,
    pool: web::Data<DbPool>,
    keypair: web::Data<Keypair>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let conn = pool
        .get_ref()
        .get()
        .expect("Could not establish a database connection");

    web::block(move || User::find(&body.email, &body.password, &conn)).then(move |res| match res {
        Err(_) => Ok(HttpResponse::Forbidden().into()),
        Ok(user) => create_token(&keypair.secret, &user)
            .map(|token| {
                let response = HttpResponse::Ok().json(LoginResponse {
                    access_token: token.to_string(),
                });
                Ok(response)
            })
            .unwrap_or_else(|_| Ok(HttpResponse::InternalServerError().into())),
    })
}

pub fn build_routes() -> actix_web::Resource {
    web::resource("").route(web::post().to_async(login))
}
