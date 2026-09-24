use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, signal},
    view::{View, component, view},
};

use super::update::update_contact;

#[component]
pub(super) async fn contact_row(
    cx: &Cx,
    id: String,
    initial_name: String,
    initial_email: String,
) -> Result<impl View> {
    let editing = signal(cx, || false);

    let saved_name = signal(cx, || initial_name.clone());
    let saved_email = signal(cx, || initial_email.clone());

    let draft_name = signal(cx, || initial_name);
    let draft_email = signal(cx, || initial_email);

    let status = signal(cx, String::new);
    let name_error = signal(cx, String::new);
    let email_error = signal(cx, String::new);

    Ok(view! {
        <tr>
            <td>
                <span :hidden=$(editing.get())>
                    $(saved_name.get())
                </span>

                <div :hidden=$(!editing.get())>
                    <input
                        class="form-control"
                        type="text"
                        autocomplete="name"
                        :value=$(draft_name.get())
                        @input=$(|e: Event| {
                            draft_name.set(e.target.value);
                        })
                    >

                    <div
                        class="invalid-feedback d-block"
                        :hidden=$(name_error.get() == "")
                    >
                        $(name_error.get())
                    </div>
                </div>
            </td>

            <td>
                <span :hidden=$(editing.get())>
                    $(saved_email.get())
                </span>

                <div :hidden=$(!editing.get())>
                    <input
                        class="form-control"
                        type="email"
                        autocomplete="email"
                        :value=$(draft_email.get())
                        @input=$(|e: Event| {
                            draft_email.set(e.target.value);
                        })
                    >

                    <div
                        class="invalid-feedback d-block"
                        :hidden=$(email_error.get() == "")
                    >
                        $(email_error.get())
                    </div>
                </div>
            </td>

            <td class="text-end">
                <div :hidden=$(editing.get())>
                    <button
                        class="btn btn-sm btn-outline-primary"
                        type="button"
                        @click=$(|_e| {
                            draft_name.set(saved_name.get());
                            draft_email.set(saved_email.get());

                            name_error.set("".to_owned());
                            email_error.set("".to_owned());
                            status.set("".to_owned());

                            editing.set(true);
                        })
                    >
                        "Edit"
                    </button>

                    <div
                        class="small text-success mt-2"
                        role="status"
                        :hidden=$(status.get() != "updated")
                    >
                        "Contact updated."
                    </div>
                </div>

                <div :hidden=$(!editing.get())>
                    <div class="d-flex justify-content-end gap-2">
                        <button
                            class="btn btn-sm btn-primary"
                            type="button"
                            :disabled=$(status.get() == "submitting")
                            @click=$(async |_e| {
                                name_error.set("".to_owned());
                                email_error.set("".to_owned());
                                status.set("submitting".to_owned());

                                let outcome = update_contact(
                                    id,
                                    draft_name.get(),
                                    draft_email.get(),
                                ).await;

                                if outcome.is_ok() {
                                    let normalized_email = outcome.unwrap();

                                    saved_name.set(
                                        draft_name.get().trim().to_owned()
                                    );
                                    saved_email.set(normalized_email);

                                    draft_name.set(saved_name.get());
                                    draft_email.set(saved_email.get());

                                    status.set("updated".to_owned());
                                    editing.set(false);
                                } else {
                                    let error = outcome.unwrap_err();

                                    if error == "validation_name" {
                                        name_error.set(
                                            "Name is required and must be 200 characters or fewer."
                                                .to_owned()
                                        );
                                        status.set("validation".to_owned());
                                    }

                                    if error == "validation_email" {
                                        email_error.set(
                                            "Enter a valid email address no longer than 320 characters."
                                                .to_owned()
                                        );
                                        status.set("validation".to_owned());
                                    }

                                    if error == "validation_both" {
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

                                    if error == "duplicate" {
                                        email_error.set(
                                            "A contact with this email address already exists."
                                                .to_owned()
                                        );
                                        status.set("validation".to_owned());
                                    }

                                    if error == "not_found" {
                                        status.set("not_found".to_owned());
                                    }

                                    if error == "error" {
                                        status.set("error".to_owned());
                                    }
                                }
                            })
                        >
                            "Save"
                        </button>

                        <button
                            class="btn btn-sm btn-outline-secondary"
                            type="button"
                            :disabled=$(status.get() == "submitting")
                            @click=$(|_e| {
                                draft_name.set(saved_name.get());
                                draft_email.set(saved_email.get());

                                name_error.set("".to_owned());
                                email_error.set("".to_owned());
                                status.set("".to_owned());

                                editing.set(false);
                            })
                        >
                            "Cancel"
                        </button>
                    </div>

                    <div
                        class="small text-secondary mt-2"
                        role="status"
                        :hidden=$(status.get() != "submitting")
                    >
                        "Saving..."
                    </div>

                    <div
                        class="small text-warning mt-2"
                        role="alert"
                        :hidden=$(status.get() != "validation")
                    >
                        "Please correct the highlighted fields."
                    </div>

                    <div
                        class="small text-warning mt-2"
                        role="alert"
                        :hidden=$(status.get() != "not_found")
                    >
                        "This contact is no longer available."
                    </div>

                    <div
                        class="small text-danger mt-2"
                        role="alert"
                        :hidden=$(status.get() != "error")
                    >
                        "The contact could not be updated."
                    </div>
                </div>
            </td>
        </tr>
    })
}
