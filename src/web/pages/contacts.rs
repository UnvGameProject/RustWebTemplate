use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::page,
    runtime::{Event, procedure, shard, signal},
    view::{View, view},
};

use crate::{
    db::models::contact::Contact,
    domain::contact::{ContactError, CreateContactInput},
};

#[procedure]
async fn create_contact(cx: &Cx, name: String, email: String) -> Result<String> {
    let input = CreateContactInput::from_untrusted(name, email);

    let input = match input.validate() {
        Ok(input) => input,
        Err(_) => return Ok("validation".to_owned()),
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

#[shard]
async fn contact_list(cx: &Cx, refresh: f64) -> Result<impl View> {
    // The refresh value itself has no business meaning.
    // Changing it causes Topcoat to re-render this shard.
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

#[page("/contacts")]
async fn contacts(cx: &Cx) -> Result<impl View> {
    let name = signal(cx, String::new);
    let email = signal(cx, String::new);
    let status = signal(cx, String::new);
    let refresh = signal(cx, || 0.0);

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

            <div class="row g-4">
                <section class="col-lg-5">
                    <div class="card shadow-sm">
                        <div class="card-body p-4">
                            <h2 class="h4 mb-3">"Create contact"</h2>

                            <p class="text-body-secondary">
                                "This form submits through a Topcoat procedure and is validated on the server before persistence."
                            </p>

                            <div
                                class="alert alert-info"
                                role="status"
                                :hidden=$(status.get() != "submitting")
                            >
                                "Creating contact..."
                            </div>

                            <div
                                class="alert alert-success"
                                role="status"
                                :hidden=$(status.get() != "created")
                            >
                                "Contact created successfully."
                            </div>

                            <div
                                class="alert alert-warning"
                                role="alert"
                                :hidden=$(status.get() != "validation")
                            >
                                "The submitted contact information is invalid."
                            </div>

                            <div
                                class="alert alert-warning"
                                role="alert"
                                :hidden=$(status.get() != "duplicate")
                            >
                                "A contact with this email address already exists."
                            </div>

                            <div
                                class="alert alert-danger"
                                role="alert"
                                :hidden=$(status.get() != "error")
                            >
                                "The contact could not be created."
                            </div>

                            <form
                                novalidate=""
                                @submit=$(async |e: Event| {
                                    e.prevent_default();

                                    status.set("submitting".to_owned());

                                    let outcome =
                                        create_contact(name.get(), email.get()).await;

                                    if outcome == "created" {
                                        name.set("".to_owned());
                                        email.set("".to_owned());
                                        refresh.increment();
                                    }

                                    status.set(outcome);
                                })
                            >
                                <div class="mb-3">
                                    <label
                                        class="form-label"
                                        for="contact-name"
                                    >
                                        "Name"
                                    </label>

                                    <input
                                        id="contact-name"
                                        class="form-control"
                                        type="text"
                                        name="name"
                                        autocomplete="name"
                                        :value=$(name.get())
                                        @input=$(|e: Event| {
                                            name.set(e.target.value);
                                        })
                                    >
                                </div>

                                <div class="mb-3">
                                    <label
                                        class="form-label"
                                        for="contact-email"
                                    >
                                        "Email"
                                    </label>

                                    <input
                                        id="contact-email"
                                        class="form-control"
                                        type="email"
                                        name="email"
                                        autocomplete="email"
                                        :value=$(email.get())
                                        @input=$(|e: Event| {
                                            email.set(e.target.value);
                                        })
                                    >
                                </div>

                                <button
                                    class="btn btn-primary"
                                    type="submit"
                                >
                                    "Create contact"
                                </button>
                            </form>
                        </div>
                    </div>
                </section>

                <section class="col-lg-7">
                    <div class="d-flex align-items-center justify-content-between mb-3">
                        <h2 class="h4 mb-0">"Existing contacts"</h2>
                    </div>

                    contact_list(refresh: $(refresh.get()))
                </section>
            </div>
        </main>
    })
}
