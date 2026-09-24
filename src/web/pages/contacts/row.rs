use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, signal},
    view::{View, component, view},
};

use super::{delete::delete_contact, update::update_contact};

#[component]
pub(super) async fn contact_row(
    cx: &Cx,
    id: String,
    initial_name: String,
    initial_email: String,
    refresh: &Signal<f64>,
) -> Result<impl View> {
    let editing = signal(cx, || false);

    let saved_name = signal(cx, || initial_name.clone());
    let saved_email = signal(cx, || initial_email.clone());

    let draft_name = signal(cx, || initial_name);
    let draft_email = signal(cx, || initial_email);

    let status = signal(cx, String::new);
    let name_error = signal(cx, String::new);
    let email_error = signal(cx, String::new);

    let confirming_delete = signal(cx, || false);
    let delete_status = signal(cx, String::new);

    let row_id = format!("contact-{id}");

    // Keep independent captured IDs for the two browser-side procedures.
    let update_id = id.clone();
    let delete_id = id;

    Ok(view! {
        <tr id=(row_id)>
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
                    <div class="d-flex justify-content-end gap-2">
                        <button
                            class="btn btn-sm btn-outline-primary"
                            type="button"
                            :disabled=$(confirming_delete.get())
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

                        <button
                            class="btn btn-sm btn-outline-danger"
                            type="button"
                            :disabled=$(confirming_delete.get())
                            @click=$(|_e| {
                                status.set("".to_owned());
                                delete_status.set("".to_owned());
                                confirming_delete.set(true);
                            })
                        >
                            "Delete"
                        </button>
                    </div>

                    <div
                        class="small text-success mt-2"
                        role="status"
                        :hidden=$(status.get() != "updated")
                    >
                        "Contact updated."
                    </div>

                    <div
                        class="border border-danger rounded-3 p-3 mt-2 text-start"
                        :hidden=$(!confirming_delete.get())
                    >
                        <div class="small fw-semibold">
                            "Delete this contact?"
                        </div>

                        <div class="small text-body-secondary mt-1">
                            "This action cannot be undone."
                        </div>

                        <div class="d-flex gap-2 mt-3">
                            <button
                                class="btn btn-sm btn-danger"
                                type="button"
                                :disabled=$(delete_status.get() == "submitting")
                                @click=$(async |_e| {
                                    delete_status.set("submitting".to_owned());

                                    let outcome =
                                        delete_contact(delete_id).await;

                                    if outcome.is_ok() {
                                        refresh.increment();
                                    } else {
                                        let error = outcome.unwrap_err();

                                        if error == "not_found" {
                                            delete_status.set(
                                                "not_found".to_owned()
                                            );
                                        }

                                        if error == "error" {
                                            delete_status.set(
                                                "error".to_owned()
                                            );
                                        }
                                    }
                                })
                            >
                                "Confirm delete"
                            </button>

                            <button
                                class="btn btn-sm btn-outline-secondary"
                                type="button"
                                :disabled=$(delete_status.get() == "submitting")
                                @click=$(|_e| {
                                    delete_status.set("".to_owned());
                                    confirming_delete.set(false);
                                })
                            >
                                "Cancel"
                            </button>
                        </div>

                        <div
                            class="small text-secondary mt-2"
                            role="status"
                            :hidden=$(delete_status.get() != "submitting")
                        >
                            "Deleting..."
                        </div>

                        <div
                            class="small text-warning mt-2"
                            role="alert"
                            :hidden=$(delete_status.get() != "not_found")
                        >
                            "This contact is no longer available."
                        </div>

                        <div
                            class="small text-danger mt-2"
                            role="alert"
                            :hidden=$(delete_status.get() != "error")
                        >
                            "The contact could not be deleted."
                        </div>
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
                                    update_id,
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
