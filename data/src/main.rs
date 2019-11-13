#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate scrypt;
#[cfg(test)]
#[macro_use]
extern crate assert_matches;
#[macro_use]
extern crate lazy_static;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware, web, App, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub mod schema;
#[cfg(test)]
pub mod test_helpers;

pub mod auth;
pub mod users;
pub mod config;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

fn get_db_connection_pool(url: &str) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool")
}

#[derive(Clone, Debug)]
struct Keypair<'a> {
    pub secret: &'a [u8],
    pub public: &'a [u8],
}

fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let pool = get_db_connection_pool(&config::DATABASE_URL);
    HttpServer::new(move || {
        let secret_key = config::get_secret_key();
        let public_key = config::get_public_key();
        let keypair = Keypair {
            secret: &secret_key,
            public: &public_key,
        };

        App::new()
            .data(pool.clone())
            .data(keypair)
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(keypair.secret)
                    .name("auth")
                    .path("/")
                    .domain(config::DOMAIN.as_str())
                    .max_age_time(chrono::Duration::days(90))
                    .secure(false),
            ))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/v1")
                    .service(web::scope("/users").service(crate::users::routes::build_routes())),
            )
    })
    .bind("127.0.0.1:3001")?
    .run()
}
