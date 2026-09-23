use std::{
    env,
    fs,
    io::{Error, ErrorKind},
};

pub(crate) struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
    user: String,
    password: String,
}

impl DatabaseConfig {
    pub(crate) fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let host = env::var("DB_HOST")?;
        let port = env::var("DB_PORT")?.parse()?;
        let name = env::var("DB_NAME")?;
        let user = env::var("DB_USER")?;
        let password_file = env::var("DB_PASSWORD_FILE")?;

        let password = fs::read_to_string(password_file)?;
        let password = password
            .trim_end_matches(|character| character == '\r' || character == '\n')
            .to_owned();

        if password.is_empty() {
            return Err(
                Error::new(
                    ErrorKind::InvalidData,
                    "database password secret is empty",
                )
                    .into(),
            );
        }

        Ok(Self {
            host,
            port,
            name,
            user,
            password,
        })
    }

    pub(crate) fn host(&self) -> &str {
        &self.host
    }

    pub(crate) fn port(&self) -> u16 {
        self.port
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn user(&self) -> &str {
        &self.user
    }

    pub(crate) fn password(&self) -> &str {
        &self.password
    }
}