use topcoat::{
    Result,
    router::{Slot, layout},
    view::{View, view},
};

use crate::web::assets::{APP_CSS, BOOTSTRAP_JS, PLAIN_CSS};

#[layout("/")]
async fn app_layout(slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" data-bs-theme="dark">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <meta name="color-scheme" content="dark light">

                <title>"Topcoat POC"</title>

                <link rel="stylesheet" href=(APP_CSS)>
                <link rel="stylesheet" href=(PLAIN_CSS)>

                <script src=(BOOTSTRAP_JS) defer=""></script>

                topcoat::runtime::script()
                topcoat::dev::script() // TODO: expose the Topcoat dev endpoint through Docker.
            </head>

            <body>
                (slot)
            </body>
        </html>
    })
}
