#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate scrypt;
#[cfg(test)]
#[macro_use]
extern crate assert_matches;
#[macro_use]
extern crate lazy_static;

use actix_web::{middleware, web, App, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub mod schema;
#[cfg(test)]
pub mod test_helpers;

pub mod auth;
pub mod config;
pub mod users;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

fn get_db_connection_pool(url: &str) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool")
}

#[derive(Clone, Debug)]
pub struct Keypair {
    pub secret: Vec<u8>,
    pub public: Vec<u8>,
}

fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let pool = get_db_connection_pool(&config::DATABASE_URL);
    let key = Keypair {
        secret: config::get_secret_key(),
        public: config::get_public_key(),
    };

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(key.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/v1")
                    .service(web::scope("/users").service(crate::users::routes::build_routes()))
                    .service(
                        web::scope("/user_tokens").service(crate::auth::routes::build_routes()),
                    ),
            )
    })
    .bind("127.0.0.1:3001")?
    .run()
}
