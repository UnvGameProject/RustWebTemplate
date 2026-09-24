use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::domain::contact::{ContactError, CreateContactInput, UpdateContactInput};

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

    let contact = Contact::create(&pool, &input)
        .await
        .expect("contact should be created");

    assert_eq!(contact.name, "Ada Lovelace");
    assert_eq!(contact.email, "ada@example.com");

    let persisted = Contact::find_by_id(&pool, contact.id).await?;

    assert_eq!(persisted, Some(contact));

    Ok(())
}

#[sqlx::test]
async fn create_maps_duplicate_email_to_contact_error(pool: PgPool) -> Result<(), sqlx::Error> {
    let first = CreateContactInput::from_untrusted("Ada Lovelace", "ada@example.com")
        .validate()
        .expect("first contact should be valid");

    Contact::create(&pool, &first)
        .await
        .expect("first contact should be created");

    let duplicate = CreateContactInput::from_untrusted("Another Ada", "ADA@example.com")
        .validate()
        .expect("duplicate contact input should still be valid");

    let error = Contact::create(&pool, &duplicate)
        .await
        .expect_err("duplicate email should be rejected");

    assert!(matches!(error, ContactError::EmailAlreadyExists));

    Ok(())
}

#[sqlx::test]
async fn update_persists_validated_contact(pool: PgPool) -> Result<(), sqlx::Error> {
    let contact = insert_contact(
        &pool,
        Uuid::new_v4(),
        "Ada Lovelace",
        "ada@example.com",
        OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
    )
    .await?;

    let input = UpdateContactInput::from_untrusted("  Grace Hopper  ", "  GRACE@Example.COM  ")
        .validate()
        .expect("updated contact should be valid");

    let updated = Contact::update(&pool, contact.id, &input)
        .await
        .expect("contact should update");

    assert_eq!(updated.id, contact.id);
    assert_eq!(updated.name, "Grace Hopper");
    assert_eq!(updated.email, "grace@example.com");
    assert_eq!(updated.created_at, contact.created_at);
    assert!(updated.updated_at > contact.updated_at);

    Ok(())
}

#[sqlx::test]
async fn update_returns_not_found_for_missing_contact(pool: PgPool) -> Result<(), sqlx::Error> {
    let input = UpdateContactInput::from_untrusted("Grace Hopper", "grace@example.com")
        .validate()
        .expect("updated contact should be valid");

    let error = Contact::update(&pool, Uuid::new_v4(), &input)
        .await
        .expect_err("missing contact should not update");

    assert!(matches!(error, ContactError::NotFound));

    Ok(())
}

#[sqlx::test]
async fn update_maps_duplicate_email_to_contact_error(pool: PgPool) -> Result<(), sqlx::Error> {
    let first = insert_contact(
        &pool,
        Uuid::new_v4(),
        "Ada Lovelace",
        "ada@example.com",
        OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
    )
    .await?;

    let second = insert_contact(
        &pool,
        Uuid::new_v4(),
        "Grace Hopper",
        "grace@example.com",
        OffsetDateTime::UNIX_EPOCH + Duration::hours(2),
    )
    .await?;

    let input = UpdateContactInput::from_untrusted("Grace Hopper", first.email)
        .validate()
        .expect("updated contact should be valid");

    let error = Contact::update(&pool, second.id, &input)
        .await
        .expect_err("duplicate email should be rejected");

    assert!(matches!(error, ContactError::EmailAlreadyExists));

    Ok(())
}

#[sqlx::test]
async fn delete_removes_existing_contact(pool: PgPool) -> Result<(), sqlx::Error> {
    let contact = insert_contact(
        &pool,
        Uuid::new_v4(),
        "Ada Lovelace",
        "ada@example.com",
        OffsetDateTime::UNIX_EPOCH + Duration::hours(1),
    )
    .await?;

    Contact::delete(&pool, contact.id)
        .await
        .expect("existing contact should be deleted");

    let persisted = Contact::find_by_id(&pool, contact.id).await?;

    assert_eq!(persisted, None);

    Ok(())
}

#[sqlx::test]
async fn delete_returns_not_found_for_missing_contact(pool: PgPool) -> Result<(), sqlx::Error> {
    let error = Contact::delete(&pool, Uuid::new_v4())
        .await
        .expect_err("missing contact should not delete");

    assert!(matches!(error, ContactError::NotFound));

    Ok(())
}
