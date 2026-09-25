use sqlx::PgPool;

use crate::domain::contact::CreateContactInput;

use super::super::Contact;

#[sqlx::test]
async fn sql_shaped_name_is_persisted_literally(pool: PgPool) -> Result<(), sqlx::Error> {
    let payload = "Robert'); DROP TABLE contacts;--";

    let input = CreateContactInput::from_untrusted(payload, "sql-shaped@example.com")
        .validate()
        .expect("SQL-shaped name should satisfy domain validation");

    let contact = Contact::create(&pool, &input)
        .await
        .expect("SQL-shaped text should persist as data");

    assert_eq!(contact.name, payload);

    let persisted = Contact::find_by_id(&pool, contact.id)
        .await?
        .expect("persisted contact should still exist");

    assert_eq!(persisted.name, payload);

    // Prove the contacts table is still functional after storing the payload.
    let second = CreateContactInput::from_untrusted("Grace Hopper", "still-alive@example.com")
        .validate()
        .expect("second contact should be valid");

    Contact::create(&pool, &second)
        .await
        .expect("contacts table should remain usable");

    assert_eq!(Contact::count(&pool).await?, 2);

    Ok(())
}

#[sqlx::test]
async fn command_shaped_name_is_persisted_literally(pool: PgPool) -> Result<(), sqlx::Error> {
    let payload = "$(rm -rf /)";

    let input = CreateContactInput::from_untrusted(payload, "command-shaped@example.com")
        .validate()
        .expect("command-shaped name should satisfy domain validation");

    let contact = Contact::create(&pool, &input)
        .await
        .expect("command-shaped text should persist as data");

    let persisted = Contact::find_by_id(&pool, contact.id)
        .await?
        .expect("persisted contact should exist");

    assert_eq!(persisted.name, payload);

    Ok(())
}
