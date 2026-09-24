use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    runtime::shard,
    view::{View, view},
};

use crate::db::models::contact::Contact;

#[shard]
pub(super) async fn contact_list(cx: &Cx, refresh: f64) -> Result<impl View> {
    let _ = refresh;

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
                                </tr>
                            </thead>

                            <tbody>
                                for contact in contact_rows {
                                    <tr>
                                        <td>(contact.name)</td>
                                        <td>(contact.email)</td>
                                    </tr>
                                }
                            </tbody>
                        </table>
                    </div>
                </div>
            }
        </div>
    })
}
