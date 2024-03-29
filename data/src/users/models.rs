use crate::schema::users;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{encode, Header};
use scrypt::{scrypt_check, scrypt_simple, ScryptParams};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum AuthenticationError {
    UserNotFound,
    IncorrectPassword,
    NoUsernameSet,
    NoPasswordSet,
    TokenGenerationError(jsonwebtoken::errors::Error),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    iat: i64,
    exp: i64,
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

    pub fn create(&self, conn: &PgConnection) -> QueryResult<User> {
        use crate::schema::users::dsl::*;
        use diesel::insert_into;

        conn.transaction(|| {
            let user = insert_into(users)
                .values(self)
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
            .create(conn)
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

    pub fn login(
        email: &str,
        password: &str,
        conn: &PgConnection,
    ) -> Result<String, AuthenticationError> {
        User::find(email, password, conn).and_then(|_| {
            let now = Utc::now().timestamp();
            let claims = Claims {
                sub: email.to_string(),
                iat: now,
                exp: now + Duration::days(90).num_seconds(),
            };

            let header = Header::default();
            encode(&header, &claims, "secret".as_ref())
                .map_err(AuthenticationError::TokenGenerationError)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use diesel::result::{DatabaseErrorKind::UniqueViolation, Error::DatabaseError};

    #[test]
    fn register_works() {
        let conn = get_test_connection();

        let user = User::register(
            "Scott Trinh",
            "scott+test@scotttrinh.com",
            "aPassword",
            &conn,
        )
        .unwrap();
        assert_eq!(user.name, "Scott Trinh");
        assert_eq!(user.email, "scott+test@scotttrinh.com");

        let expected_user = User::find("scott+test@scotttrinh.com", "aPassword", &conn).unwrap();
        assert_eq!(user.id, expected_user.id);
        assert_eq!(user.name, expected_user.name);
        assert_eq!(user.email, expected_user.email);
    }

    #[test]
    fn register_conflict() {
        let conn = get_test_connection();

        User::register(
            "Scott Trinh",
            "scott+test@scotttrinh.com",
            "aPassword",
            &conn,
        )
        .unwrap();

        let conflict = User::register(
            "Scott Trinh",
            "scott+test@scotttrinh.com",
            "aPassword",
            &conn,
        );
        assert_matches!(
            conflict,
            Err(AuthenticationError::DatabaseError(DatabaseError(UniqueViolation, _)))
        );
    }

    #[test]
    fn find_wrong_password() {
        let conn = get_test_connection();

        User::register(
            "Scott Trinh",
            "scott+test@scotttrinh.com",
            "aPassword",
            &conn,
        )
        .unwrap();

        let wrong_password = User::find("scott+test@scotttrinh.com", "wrongPassword", &conn);
        assert_matches!(wrong_password, Err(AuthenticationError::IncorrectPassword));
    }

    #[test]
    fn find_miss() {
        let conn = get_test_connection();

        User::register(
            "Scott Trinh",
            "scott+test@scotttrinh.com",
            "aPassword",
            &conn,
        )
        .unwrap();

        let wrong_email = User::find("scott+missing@scotttrinh.com", "aPassword", &conn);
        assert_matches!(wrong_email, Err(AuthenticationError::UserNotFound));
    }
}
