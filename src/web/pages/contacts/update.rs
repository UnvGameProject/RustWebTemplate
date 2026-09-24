use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    runtime::procedure,
};
use uuid::Uuid;

use crate::{
    db::models::contact::Contact,
    domain::contact::{ContactError, UpdateContactInput},
};

use super::validation::validation_outcome;

#[procedure]
pub(super) async fn update_contact(
    cx: &Cx,
    id: String,
    name: String,
    email: String,
) -> Result<std::result::Result<String, String>> {
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(Err("not_found".to_owned()));
        }
    };

    let input = UpdateContactInput::from_untrusted(name, email);

    let input = match input.validate() {
        Ok(input) => input,
        Err(report) => {
            return Ok(Err(validation_outcome(&report).to_owned()));
        }
    };

    let pool = app_context::<PgPool>(cx);

    match Contact::update(pool, id, &input).await {
        Ok(contact) => Ok(Ok(contact.email)),

        Err(ContactError::EmailAlreadyExists) => Ok(Err("duplicate".to_owned())),

        Err(ContactError::NotFound) => Ok(Err("not_found".to_owned())),

        Err(error) => {
            eprintln!("contact update failed: {error:?}");
            Ok(Err("error".to_owned()))
        }
    }
}
