use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::signal,
    view::{View, view},
};

#[page("/")]
async fn home(cx: &Cx) -> Result<impl View> {
    let details_open = signal(cx, || false);

    Ok(view! {
        <main class="poc-shell d-flex align-items-center py-5">
            <div class="container py-5">
                <div class="d-flex flex-wrap align-items-center gap-2 mb-4">
                    <span class="badge rounded-pill text-bg-success">
                        "POC 0 - Rendering"
                    </span>

                    <span class="text-secondary small">
                        "Topcoat 0.8.1 | Rust 1.98 | Bootstrap 5.3.8 | Docker"
                    </span>
                </div>

                <div class="row g-5 align-items-center">
                    <section class="col-lg-8">
                        <p class="poc-eyebrow text-info fw-semibold text-uppercase small mb-3">
                            "Laravel-style product development, tested in Rust"
                        </p>

                        <h1 class="display-3 fw-bold text-white">
                            "Can Topcoat make a Rust frontend productive enough to replace Laravel + Livewire?"
                        </h1>

                        <p class="lead text-secondary mt-4">
                            "This is the controlled test bed. Bootstrap is compiled from SCSS, ordinary CSS is loaded separately, and Topcoat remains responsible for SSR, routing, assets, and Rust-authored reactivity."
                        </p>

                        <div class="d-flex flex-wrap gap-3 mt-4">
                            <button
                                type="button"
                                class="btn btn-primary btn-lg"
                                @click=$(|_e| details_open.set(!details_open.get()))
                            >
                                "Toggle Topcoat reactivity"
                            </button>

                            <button
                                type="button"
                                class="btn btn-outline-light btn-lg"
                                data-bs-toggle="collapse"
                                data-bs-target="#bootstrap-check"
                                aria-expanded="false"
                                aria-controls="bootstrap-check"
                            >
                                "Toggle Bootstrap JS"
                            </button>
                        </div>

                        <div
                            class="alert alert-info mt-4"
                            role="status"
                            :hidden=$(!details_open.get())
                        >
                            <strong>"Topcoat runtime: "</strong>
                            "this state change was authored in Rust and did not require a server round-trip."
                        </div>

                        <div class="collapse mt-3" id="bootstrap-check">
                            <div class="card card-body">
                                <strong>"Bootstrap JavaScript is active."</strong>

                                <span class="text-secondary">
                                    " The bundle is pinned at build time and served locally by Topcoat rather than loaded by the browser from a CDN."
                                </span>
                            </div>
                        </div>
                    </section>

                    <aside class="col-lg-4">
                        <div class="poc-panel card shadow-lg">
                            <div class="card-body p-4">
                                <h2 class="h4 card-title">
                                    "POC acceptance gates"
                                </h2>

                                <div class="vstack gap-3 mt-4">
                                    <div class="border rounded-3 p-3">
                                        <div class="fw-semibold">
                                            "1. Frontend velocity"
                                        </div>

                                        <div class="small text-secondary mt-1">
                                            "Layouts, components, forms, reactive UX, CSS/SCSS freedom, and maintainability."
                                        </div>
                                    </div>

                                    <div class="poc-security-note border rounded-3 p-3">
                                        <div class="fw-semibold">
                                            "2. Application security"
                                        </div>

                                        <div class="small text-secondary mt-1">
                                            "Laravel-grade defaults or explicit, tested Rust replacements for every missing guardrail."
                                        </div>
                                    </div>

                                    <div class="border rounded-3 p-3">
                                        <div class="fw-semibold">
                                            "3. Operational simplicity"
                                        </div>

                                        <div class="small text-secondary mt-1">
                                            "Reproducible Docker builds, minimal runtime dependencies, logging, tests, and deployability."
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </aside>
                </div>
            </div>
        </main>
    })
}