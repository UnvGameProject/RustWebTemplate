use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::page,
    view::{View, view},
};

use crate::db::models::contact::Contact;

#[page("/contacts")]
async fn contacts(cx: &Cx) -> Result<impl View> {
    let pool = app_context::<PgPool>(cx);
    let contacts = Contact::find_all(pool).await?;

    Ok(view! {
        <main class="container py-5">
            <div class="d-flex justify-content-between align-items-center mb-4">
                <div>
                    <h1 class="mb-1">"Contacts"</h1>
                    <p class="text-body-secondary mb-0">
                        "Topcoat + SQLx contact workflow"
                    </p>
                </div>

                <a class="btn btn-outline-secondary" href="/">
                    "Home"
                </a>
            </div>

            if contacts.is_empty() {
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
                                for contact in contacts {
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
        </main>
    })
}
