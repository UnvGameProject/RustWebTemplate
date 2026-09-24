use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    runtime::procedure,
};
use uuid::Uuid;

use crate::{db::models::contact::Contact, domain::contact::ContactError};

#[procedure]
pub(super) async fn delete_contact(
    cx: &Cx,
    id: String,
) -> Result<std::result::Result<String, String>> {
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(Err("not_found".to_owned()));
        }
    };

    let pool = app_context::<PgPool>(cx);

    match Contact::delete(pool, id).await {
        Ok(()) => Ok(Ok("deleted".to_owned())),

        Err(ContactError::NotFound) => Ok(Err("not_found".to_owned())),

        Err(error) => {
            eprintln!("contact deletion failed: {error:?}");
            Ok(Err("error".to_owned()))
        }
    }
}
