use diesel::prelude::*;
use diesel_migrations::run_pending_migrations;
use dotenv;

pub fn get_test_connection() -> PgConnection {
    let database_url = dotenv::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let conn = PgConnection::establish(&database_url).unwrap();
    run_pending_migrations(&conn).unwrap();
    conn.begin_test_transaction().unwrap();
    conn
}
