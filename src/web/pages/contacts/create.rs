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

pub(super) fn validation_outcome(report: &garde::Report) -> &'static str {
    let name_invalid = garde::select!(report, name).next().is_some();
    let email_invalid = garde::select!(report, email).next().is_some();

    match (name_invalid, email_invalid) {
        (true, true) => "validation_both",
        (true, false) => "validation_name",
        (false, true) => "validation_email",
        (false, false) => "validation",
    }
}

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
