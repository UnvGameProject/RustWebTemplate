use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    runtime::{Signal, shard},
    view::{View, view},
};

use crate::db::models::contact::Contact;

use super::row::contact_row;

#[shard]
pub(super) async fn contact_list(cx: &Cx, refresh: Signal<f64>) -> Result<impl View> {
    // Track this signal on the server. Incrementing it causes only this shard
    // to re-render.
    let _ = refresh.get();

    let pool = app_context::<PgPool>(cx);
    let contact_rows = Contact::find_all(pool).await?;

    Ok(view! {
        <div>
            if contact_rows.is_empty() {
                <div class="alert alert-secondary" role="status">
                    "No contacts have been created yet."
                </div>
            } else {
                <div class="card shadow-sm">
                    <div class="table-responsive">
                        <table class="table table-striped align-middle mb-0">
                            <thead>
                                <tr>
                                    <th scope="col">"Name"</th>
                                    <th scope="col">"Email"</th>
                                    <th class="text-end" scope="col">"Actions"</th>
                                </tr>
                            </thead>

                            <tbody>
                                #[key(contact.id.to_string())]
                                for contact in contact_rows {
                                    contact_row(
                                        id: contact.id.to_string(),
                                        initial_name: contact.name,
                                        initial_email: contact.email,
                                        refresh: &refresh,
                                    )
                                }
                            </tbody>
                        </table>
                    </div>
                </div>
            }
        </div>
    })
}
