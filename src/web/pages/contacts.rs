use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::{Event, signal},
    view::{View, view},
};

use self::{create::create_contact, list::contact_list};

mod create;
mod list;
mod row;
#[cfg(test)]
mod tests;
mod update;
mod validation;

#[page("/contacts")]
async fn contacts(cx: &Cx) -> Result<impl View> {
    let name = signal(cx, String::new);
    let email = signal(cx, String::new);
    let status = signal(cx, String::new);
    let name_error = signal(cx, String::new);
    let email_error = signal(cx, String::new);
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
                                "Please correct the highlighted fields."
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

                                    name_error.set("".to_owned());
                                    email_error.set("".to_owned());
                                    status.set("submitting".to_owned());

                                    let outcome =
                                        create_contact(name.get(), email.get()).await;

                                    if outcome == "validation_name" {
                                        name_error.set(
                                            "Name is required and must be 200 characters or fewer."
                                                .to_owned()
                                        );
                                        status.set("validation".to_owned());
                                    }

                                    if outcome == "validation_email" {
                                        email_error.set(
                                            "Enter a valid email address no longer than 320 characters."
                                                .to_owned()
                                        );
                                        status.set("validation".to_owned());
                                    }

                                    if outcome == "validation_both" {
                                        name_error.set(
                                            "Name is required and must be 200 characters or fewer."
                                                .to_owned()
                                        );
                                        email_error.set(
                                            "Enter a valid email address no longer than 320 characters."
                                                .to_owned()
                                        );
                                        status.set("validation".to_owned());
                                    }

                                    if outcome == "duplicate" {
                                        email_error.set(
                                            "A contact with this email address already exists."
                                                .to_owned()
                                        );
                                        status.set("validation".to_owned());
                                    }

                                    if outcome == "created" {
                                        name.set("".to_owned());
                                        email.set("".to_owned());
                                        refresh.increment();
                                        status.set("created".to_owned());
                                    }

                                    if outcome == "error" {
                                        status.set("error".to_owned());
                                    }
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

                                    <div
                                        class="invalid-feedback d-block"
                                        :hidden=$(name_error.get() == "")
                                    >
                                        $(name_error.get())
                                    </div>
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

                                    <div
                                        class="invalid-feedback d-block"
                                        :hidden=$(email_error.get() == "")
                                    >
                                        $(email_error.get())
                                    </div>
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
