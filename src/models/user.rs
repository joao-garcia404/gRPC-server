use chrono::Utc;
use uuid::Uuid;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

#[derive(Debug)]
pub struct Password {
    pub value: String,
}

impl Password {
    pub fn new(raw: String) -> Password {
        Password { value: raw }
    }

    pub fn get_hashed_value(&mut self) -> Result<(), String> {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();
        let hash_result = argon2.hash_password(self.value.as_bytes(), &salt);

        match hash_result {
            Ok(hashed) => {
                self.value = hashed.to_string();
                Ok(())
            }
            Err(_hash_error) => Err("Error at hashing password".to_owned()),
        }
    }
}

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub password: Password,
    pub created_at: String,
}

impl User {
    pub fn new(name: String, email: String, raw_password: String) -> Result<User, String> {
        let mut user = User {
            id: Uuid::new_v4().to_string(),
            name,
            email,
            password: Password::new(raw_password),
            created_at: Utc::now().to_rfc3339(),
        };

        user.password.get_hashed_value()?;

        Ok(user)
    }

    pub fn from_db(
        id: String,
        name: String,
        email: String,
        password: String,
        created_at: String,
    ) -> Result<User, String> {
        let user_id = Uuid::try_parse(id.as_str())
            .map_err(|_err| "Failed to parse user_id as UUID".to_string())?
            .to_string();

        let user = User {
            id: user_id,
            name,
            email,
            password: Password::new(password),
            created_at,
        };

        Ok(user)
    }
}
