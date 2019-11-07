#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate scrypt;
#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use actix_web::{middleware, web, App, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub mod schema;
#[cfg(test)]
pub mod test_helpers;

pub mod users;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

fn get_db_connection_pool(url: &str) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool")
}

fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = get_db_connection_pool(&database_url);

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/v1")
                    .service(web::scope("/users").service(crate::users::routes::build_routes())),
            )
    })
    .bind("127.0.0.1:3001")?
    .run()
}
