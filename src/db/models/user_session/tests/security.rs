use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

use crate::{
    auth::password::hash_password,
    db::models::{user::User, user_session::UserSession},
    domain::user::CreateUserInput,
};

async fn create_user(pool: &PgPool, email: &str) -> User {
    let input = CreateUserInput::from_untrusted(email)
        .validate()
        .expect("user input should be valid");

    User::create(
        pool,
        &input,
        hash_password("correct horse battery staple").expect("password should hash"),
    )
    .await
    .expect("user should be created")
}

#[sqlx::test]
async fn deleting_user_cascades_session_records(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "ada@example.com").await;
    let token_hash = [8_u8; 32];

    UserSession::create(
        &pool,
        &token_hash,
        user.id,
        OffsetDateTime::now_utc() + Duration::hours(1),
    )
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM public.users
        WHERE id = $1
        "#,
        user.id,
    )
    .execute(&pool)
    .await?;

    assert!(
        UserSession::find_active(&pool, &token_hash)
            .await?
            .is_none()
    );

    Ok(())
}

#[sqlx::test]
async fn debug_output_redacts_token_hash(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "ada@example.com").await;
    let token_hash = [9_u8; 32];

    let session = UserSession::create(
        &pool,
        &token_hash,
        user.id,
        OffsetDateTime::now_utc() + Duration::hours(1),
    )
    .await?;

    let debug = format!("{session:?}");
    let raw_debug = format!("{token_hash:?}");

    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains(&raw_debug));

    Ok(())
}
