use serde::Serialize;
use crate::schema::users;
use diesel::prelude::*;
use scrypt::{scrypt_check, scrypt_simple, ScryptParams};

#[derive(Debug)]
pub enum AuthenticationError {
    UserNotFound,
    IncorrectPassword,
    NoUsernameSet,
    NoPasswordSet,
    EnvironmentError(dotenv::Error),
    ScryptCheckError(scrypt::errors::CheckError),
    ScryptParamsError(scrypt::errors::InvalidParams),
    ScryptError(std::io::Error),
    DatabaseError(diesel::result::Error),
}

#[derive(Queryable, Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Queryable)]
struct UserWithPassword {
    pub user: User,
    pub password_hash: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
}

impl<'a> NewUser<'a> {
    pub fn new(name: &'a str, email: &'a str, password_hash: &'a str) -> Self {
        NewUser {
            name,
            email,
            password_hash,
        }
    }

    pub fn create_or_update(&self, conn: &PgConnection) -> QueryResult<User> {
        use crate::schema::users::dsl::*;
        use diesel::insert_into;
        use diesel::pg::upsert::excluded;

        conn.transaction(|| {
            let user = insert_into(users)
                .values(self)
                .on_conflict(email)
                .do_update()
                .set((
                    name.eq(excluded(name)),
                    email.eq(excluded(email)),
                    password_hash.eq(excluded(password_hash)),
                ))
                .returning((id, name, email))
                .get_result::<User>(conn)?;

            Ok(user)
        })
    }
}

impl User {
    pub fn register(
        name: &str,
        email: &str,
        password: &str,
        conn: &PgConnection,
    ) -> Result<Self, AuthenticationError> {
        let params = ScryptParams::new(15, 8, 1).unwrap();
        let password_hash =
            scrypt_simple(password, &params).map_err(AuthenticationError::ScryptError)?;
        NewUser::new(name, email, &password_hash)
            .create_or_update(conn)
            .map_err(AuthenticationError::DatabaseError)
    }

    pub fn find(
        email: &str,
        password: &str,
        conn: &PgConnection,
    ) -> Result<Self, AuthenticationError> {
        let user_and_password = users::table
            .filter(users::email.eq(email))
            .select(((users::id, users::name, users::email), users::password_hash))
            .first::<UserWithPassword>(conn)
            .optional()
            .map_err(AuthenticationError::DatabaseError)?;

        match user_and_password {
            None => Err(AuthenticationError::UserNotFound),
            Some(with_password) => scrypt_check(password, &with_password.password_hash)
                .map(|_| with_password.user)
                .map_err(|_| AuthenticationError::IncorrectPassword),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn register_works() {
        let conn = get_test_connection();

        let user =
            User::register("Scott Trinh", "scott@scotttrinh.com", "aPassword", &conn).unwrap();
        assert_eq!(user.name, "Scott Trinh");
        assert_eq!(user.email, "scott@scotttrinh.com");

        let expected_user = User::find("scott@scotttrinh.com", "aPassword", &conn).unwrap();
        assert_eq!(user.id, expected_user.id);
        assert_eq!(user.name, expected_user.name);
        assert_eq!(user.email, expected_user.email);

        let wrong_password = User::find("scott@scotttrinh.com", "wrongPassword", &conn);
        assert_matches!(wrong_password, Err(AuthenticationError::IncorrectPassword));

        let wrong_email = User::find("scott+1@scotttrinh.com", "aPassword", &conn);
        assert_matches!(wrong_email, Err(AuthenticationError::UserNotFound));
    }
}
