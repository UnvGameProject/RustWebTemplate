use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    runtime::procedure,
};

use crate::{
    db::models::contact::Contact,
    domain::contact::{ContactError, CreateContactInput},
};

use super::validation::validation_outcome;

#[procedure]
pub(super) async fn create_contact(cx: &Cx, name: String, email: String) -> Result<String> {
    let input = CreateContactInput::from_untrusted(name, email);

    let input = match input.validate() {
        Ok(input) => input,
        Err(report) => {
            return Ok(validation_outcome(&report).to_owned());
        }
    };

    let pool = app_context::<PgPool>(cx);

    match Contact::create(pool, &input).await {
        Ok(_) => Ok("created".to_owned()),

        Err(ContactError::EmailAlreadyExists) => Ok("duplicate".to_owned()),

        Err(error) => {
            eprintln!("contact creation failed: {error:?}");
            Ok("error".to_owned())
        }
    }
}
