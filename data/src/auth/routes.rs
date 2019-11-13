use actix_identity::Identity;
use actix_web::{cookie::Cookie, web, Error, HttpResponse};
use csrf::{AesGcmCsrfProtection, CsrfProtection};
use futures::Future;
use serde::Deserialize;

use crate::auth::token::create_token;
use crate::users::models::User;

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

pub fn login(
    body: web::Json<LoginRequest>,
    identity: Identity,
    pool: web::Data<crate::DbPool>,
    csrf_gen: web::Data<AesGcmCsrfProtection>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let conn = pool
        .get_ref()
        .get()
        .expect("Could not establish a database connection");

    let (csrf_token, csrf_cookie) = csrf_gen
        .get_ref()
        .generate_token_pair(None, 300)
        .expect("Could not generate a CSRF token");

    web::block(move || User::find(&body.email, &body.password, &conn)).then(move |res| match res {
        Err(_) => Ok(HttpResponse::Forbidden().into()),
        Ok(user) => create_token(&user)
            .map(|token| {
                identity.remember(token);

                let response = HttpResponse::Ok()
                    .cookie(
                        Cookie::build("csrf", csrf_cookie.b64_string())
                            .secure(false)
                            .http_only(true)
                            .finish(),
                    )
                    .header("X-CSRF-TOKEN", csrf_token.b64_string())
                    .finish();
                Ok(response)
            })
            .unwrap_or_else(|_| Ok(HttpResponse::InternalServerError().into())),
    })
}
