use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::domain::contact::CreateContactInput;

use super::super::Contact;

async fn insert_contact(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    email: &str,
    created_at: OffsetDateTime,
) -> Result<Contact, sqlx::Error> {
    let updated_at = created_at;

    sqlx::query(
        r#"
        INSERT INTO public.contacts (
            id,
            name,
            email,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(email)
    .bind(created_at)
    .bind(updated_at)
    .execute(pool)
    .await?;

    Ok(Contact {
        id,
        name: name.to_owned(),
        email: email.to_owned(),
        created_at,
        updated_at,
    })
}

#[sqlx::test]
async fn find_by_id_returns_none_when_contact_does_not_exist(
    pool: PgPool,
) -> Result<(), sqlx::Error> {
    let contact = Contact::find_by_id(&pool, Uuid::new_v4()).await?;

    assert!(contact.is_none());

    Ok(())
}

#[sqlx::test]
async fn find_by_id_returns_persisted_contact(pool: PgPool) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();

    let created_at = OffsetDateTime::UNIX_EPOCH + Duration::hours(1);

    let expected = insert_contact(&pool, id, "Ada Lovelace", "ada@example.com", created_at).await?;

    let actual = Contact::find_by_id(&pool, id).await?;

    assert_eq!(actual, Some(expected));

    Ok(())
}

#[sqlx::test]
async fn count_returns_number_of_contacts(pool: PgPool) -> Result<(), sqlx::Error> {
    assert_eq!(Contact::count(&pool).await?, 0);

    insert_contact(
        &pool,
        Uuid::new_v4(),
        "Ada Lovelace",
        "ada@example.com",
        OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
    )
    .await?;

    insert_contact(
        &pool,
        Uuid::new_v4(),
        "Grace Hopper",
        "grace@example.com",
        OffsetDateTime::UNIX_EPOCH + Duration::hours(2),
    )
    .await?;

    assert_eq!(Contact::count(&pool).await?, 2);

    Ok(())
}

#[sqlx::test]
async fn find_all_returns_contacts_in_deterministic_order(pool: PgPool) -> Result<(), sqlx::Error> {
    let older = insert_contact(
        &pool,
        Uuid::new_v4(),
        "Ada Lovelace",
        "ada@example.com",
        OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
    )
    .await?;

    let newer = insert_contact(
        &pool,
        Uuid::new_v4(),
        "Grace Hopper",
        "grace@example.com",
        OffsetDateTime::UNIX_EPOCH + Duration::hours(2),
    )
    .await?;

    let contacts = Contact::find_all(&pool).await?;

    assert_eq!(contacts, vec![newer, older]);

    Ok(())
}

#[sqlx::test]
async fn create_persists_validated_contact(pool: PgPool) -> Result<(), sqlx::Error> {
    let input = CreateContactInput::from_untrusted("  Ada Lovelace  ", "  ADA@Example.COM  ")
        .validate()
        .expect("contact input should be valid");

    let contact = Contact::create(&pool, &input).await?;

    assert_eq!(contact.name, "Ada Lovelace");
    assert_eq!(contact.email, "ada@example.com");

    let persisted = Contact::find_by_id(&pool, contact.id).await?;

    assert_eq!(persisted, Some(contact));

    Ok(())
}
