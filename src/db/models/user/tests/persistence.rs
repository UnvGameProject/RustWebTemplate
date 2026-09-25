use sqlx::PgPool;

use crate::{
    auth::password::{hash_password, verify_password},
    domain::user::{CreateUserInput, UserError},
};

use super::super::User;

#[sqlx::test]
async fn create_persists_validated_user(pool: PgPool) -> Result<(), sqlx::Error> {
    let input = CreateUserInput::from_untrusted("  ADA@Example.COM  ")
        .validate()
        .expect("user input should be valid");

    let password_hash =
        hash_password("correct horse battery staple").expect("password should hash");

    let user = User::create(&pool, &input, password_hash)
        .await
        .expect("user should be created");

    assert_eq!(user.email, "ada@example.com");

    let persisted = User::find_by_id(&pool, user.id).await?;

    assert_eq!(persisted, Some(user));

    Ok(())
}

#[sqlx::test]
async fn create_maps_duplicate_email_to_user_error(pool: PgPool) -> Result<(), sqlx::Error> {
    let first = CreateUserInput::from_untrusted("ada@example.com")
        .validate()
        .expect("first user input should be valid");

    User::create(
        &pool,
        &first,
        hash_password("first password").expect("first password should hash"),
    )
    .await
    .expect("first user should be created");

    let duplicate = CreateUserInput::from_untrusted("ADA@example.com")
        .validate()
        .expect("duplicate input should still be valid");

    let error = User::create(
        &pool,
        &duplicate,
        hash_password("second password").expect("second password should hash"),
    )
    .await
    .expect_err("duplicate email should be rejected");

    assert!(matches!(error, UserError::EmailAlreadyExists));

    Ok(())
}

#[sqlx::test]
async fn credentials_can_be_loaded_by_email(pool: PgPool) -> Result<(), sqlx::Error> {
    let input = CreateUserInput::from_untrusted("ada@example.com")
        .validate()
        .expect("user input should be valid");

    let user = User::create(
        &pool,
        &input,
        hash_password("correct horse battery staple").expect("password should hash"),
    )
    .await
    .expect("user should be created");

    let credentials = User::find_credentials_by_email(&pool, "ADA@EXAMPLE.COM")
        .await?
        .expect("credentials should exist");

    assert_eq!(credentials.user(), &user);

    assert!(
        verify_password("correct horse battery staple", credentials.password_hash(),)
            .expect("stored password hash should verify")
    );

    Ok(())
}

#[sqlx::test]
async fn credential_lookup_returns_none_for_unknown_email(pool: PgPool) -> Result<(), sqlx::Error> {
    let credentials = User::find_credentials_by_email(&pool, "missing@example.com").await?;

    assert!(credentials.is_none());

    Ok(())
}
