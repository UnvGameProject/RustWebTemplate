use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

use crate::{
    auth::password::hash_password,
    db::models::{user::User, user_session::UserSession},
    domain::user::CreateUserInput,
};

fn assert_same_database_timestamp(actual: OffsetDateTime, expected: OffsetDateTime) {
    let difference_nanoseconds =
        (actual.unix_timestamp_nanos() - expected.unix_timestamp_nanos()).abs();

    assert!(
        difference_nanoseconds < 1_000,
        "timestamps differ beyond database microsecond precision: \
         actual={actual}, expected={expected}",
    );
}

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
async fn create_persists_session(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "ada@example.com").await;
    let token_hash = [1_u8; 32];
    let expires_at = OffsetDateTime::now_utc() + Duration::hours(1);

    let session = UserSession::create(&pool, &token_hash, user.id, expires_at).await?;

    assert_eq!(session.token_hash(), &token_hash);
    assert_eq!(session.user_id(), user.id);
    assert_same_database_timestamp(session.expires_at(), expires_at);

    Ok(())
}

#[sqlx::test]
async fn find_active_returns_unexpired_session(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "ada@example.com").await;
    let token_hash = [2_u8; 32];

    UserSession::create(
        &pool,
        &token_hash,
        user.id,
        OffsetDateTime::now_utc() + Duration::hours(1),
    )
    .await?;

    let session = UserSession::find_active(&pool, &token_hash)
        .await?
        .expect("unexpired session should resolve");

    assert_eq!(session.user_id(), user.id);

    Ok(())
}

#[sqlx::test]
async fn find_active_rejects_expired_session(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "ada@example.com").await;
    let token_hash = [3_u8; 32];

    UserSession::create(
        &pool,
        &token_hash,
        user.id,
        OffsetDateTime::now_utc() - Duration::hours(1),
    )
    .await?;

    let session = UserSession::find_active(&pool, &token_hash).await?;

    assert!(session.is_none());

    Ok(())
}

#[sqlx::test]
async fn update_expiry_refreshes_session(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "ada@example.com").await;
    let token_hash = [4_u8; 32];

    let initial_expiry = OffsetDateTime::now_utc() + Duration::hours(1);

    UserSession::create(&pool, &token_hash, user.id, initial_expiry).await?;

    let refreshed_expiry = OffsetDateTime::now_utc() + Duration::hours(2);

    let session = UserSession::update_expiry(&pool, &token_hash, refreshed_expiry)
        .await?
        .expect("existing session should refresh");

    assert_eq!(session.token_hash(), &token_hash);
    assert_same_database_timestamp(session.expires_at(), refreshed_expiry);
    assert!(session.updated_at() >= session.created_at());

    Ok(())
}

#[sqlx::test]
async fn replace_token_revokes_old_hash(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "ada@example.com").await;

    let revoked = [5_u8; 32];
    let replacement = [6_u8; 32];

    UserSession::create(
        &pool,
        &revoked,
        user.id,
        OffsetDateTime::now_utc() + Duration::hours(1),
    )
    .await?;

    let replacement_expiry = OffsetDateTime::now_utc() + Duration::hours(2);

    let session = UserSession::replace_token(&pool, &revoked, &replacement, replacement_expiry)
        .await?
        .expect("existing session should rotate");

    assert_eq!(session.token_hash(), &replacement);
    assert_eq!(session.user_id(), user.id);
    assert_same_database_timestamp(session.expires_at(), replacement_expiry);

    assert!(UserSession::find_active(&pool, &revoked).await?.is_none());

    assert!(
        UserSession::find_active(&pool, &replacement)
            .await?
            .is_some()
    );

    Ok(())
}

#[sqlx::test]
async fn delete_removes_session(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "ada@example.com").await;
    let token_hash = [7_u8; 32];

    UserSession::create(
        &pool,
        &token_hash,
        user.id,
        OffsetDateTime::now_utc() + Duration::hours(1),
    )
    .await?;

    assert!(UserSession::delete(&pool, &token_hash).await?);

    assert!(!UserSession::delete(&pool, &token_hash).await?);

    assert!(
        UserSession::find_active(&pool, &token_hash)
            .await?
            .is_none()
    );

    Ok(())
}
