use sqlx::PgPool;

use crate::{
    auth::password::{PasswordHash, hash_password, verify_password},
    domain::user::CreateUserInput,
};

use super::super::User;

#[sqlx::test]
async fn password_is_persisted_only_as_argon2_hash(pool: PgPool) -> Result<(), sqlx::Error> {
    let raw_password = "correct horse battery staple";

    let input = CreateUserInput::from_untrusted("ada@example.com")
        .validate()
        .expect("user input should be valid");

    let user = User::create(
        &pool,
        &input,
        hash_password(raw_password).expect("password should hash"),
    )
    .await
    .expect("user should be created");

    let stored = sqlx::query_scalar!(
        r#"
        SELECT password_hash
        FROM public.users
        WHERE id = $1
        "#,
        user.id,
    )
    .fetch_one(&pool)
    .await?;

    assert_ne!(stored, raw_password);
    assert!(stored.starts_with("$argon2id$"));

    let stored = PasswordHash::from_stored(stored);

    assert!(verify_password(raw_password, &stored).expect("persisted hash should verify"));

    Ok(())
}

#[sqlx::test]
async fn credentials_debug_output_redacts_password_hash(pool: PgPool) -> Result<(), sqlx::Error> {
    let input = CreateUserInput::from_untrusted("ada@example.com")
        .validate()
        .expect("user input should be valid");

    User::create(
        &pool,
        &input,
        hash_password("correct horse battery staple").expect("password should hash"),
    )
    .await
    .expect("user should be created");

    let credentials = User::find_credentials_by_email(&pool, "ada@example.com")
        .await?
        .expect("credentials should exist");

    let debug = format!("{credentials:?}");

    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("$argon2id$"));
    assert!(!debug.contains("correct horse battery staple"));

    Ok(())
}
