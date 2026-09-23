# Topcoat POC

A security-first, Dockerized proof of concept for evaluating **Topcoat** as a Rust frontend/full-stack framework against a mature **Laravel + Livewire-style** development baseline.

This repository is also intended to become a reusable GitHub template for future Topcoat applications if the framework proves suitable.

## Purpose

The goal is not simply to prove that Rust can render a web application.

The POC is evaluating whether Topcoat can provide a productive, maintainable, and secure application-development experience while allowing an existing Rust-heavy backend to remain in Rust.

Laravel and mature industry implementations are used as reference points whenever Topcoat does not yet provide an equivalent built-in capability.

The evaluation prioritizes:

1. **Security**
2. **Organization and maintainability**
3. **Developer experience**
4. **Framework maturity**
5. **Operational simplicity**
6. **Frontend flexibility**
7. **Performance**

## Getting started

### Prerequisites

Install:

- Docker with Docker Compose
- Git
- Rustup
- Rust 1.98.1, or allow `rust-toolchain.toml` to select/install it automatically
- OpenSSL for generating the local PostgreSQL development password

The project uses Docker for the application runtime, frontend asset build, and PostgreSQL. A host Rust toolchain is also supported for IDE integration, formatting, linting, and local `cargo check`.

### Clone the repository

```bash
git clone <your-repository-url>
cd topcoat_poc
```

If this repository is being used as a GitHub template, create a new repository from the template first, then clone the new repository.

### Create the local development secret

The preferred development configuration uses Docker Compose secrets rather than a `.env` file.

Create the local secrets directory and PostgreSQL password:

```bash
mkdir -p .secrets
chmod 700 .secrets

openssl rand -hex 24 > .secrets/postgres_password
chmod 600 .secrets/postgres_password
```

The `.secrets/` directory is ignored by Git and must never be committed.

### Build and start the application

```bash
docker compose up --build
```

Or, after the images have already been built:

```bash
docker compose up -d
```

Check service status:

```bash
docker compose ps
```

The application should be available at:

```text
http://localhost:3000
```

The development port is intentionally bound to `127.0.0.1`, so it is not exposed to the LAN by default.

### Verify the Rust toolchain

On the host:

```bash
rustup show
rustc --version
cargo --version
```

The repository pins the development toolchain with `rust-toolchain.toml`.

Verify the application locally:

```bash
cargo check --locked
```

Verify the Docker build environment:

```bash
docker compose exec app rustc --version
docker compose exec app cargo check --locked
```

### Verify PostgreSQL

Once the PostgreSQL service is present and started:

```bash
docker compose exec postgres sh -c \
  'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT current_database(), current_user, version();"'
```

Verify that the application container can read its Docker secret without printing the password:

```bash
docker compose exec app sh -c \
  'test -r /run/secrets/postgres_password && echo "postgres_password secret: readable"'
```

### Development logs

Follow the Topcoat application logs:

```bash
docker compose logs -f app
```

Follow only recent application logs:

```bash
docker compose logs --since=1m -f app
```

Follow frontend asset-builder logs:

```bash
docker compose logs -f assets
```

### Formatting and linting

Check formatting:

```bash
cargo fmt --check
```

Run Clippy with warnings treated as errors:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Run the same checks in Docker when needed:

```bash
docker compose exec app cargo fmt --check
docker compose exec app cargo clippy --all-targets --all-features -- -D warnings
```

### Stop the project

```bash
docker compose down
```

To also remove project data volumes:

```bash
docker compose down -v
```

Use `-v` deliberately: it removes named volumes, including PostgreSQL development data and Cargo/Node caches.

## Current status

### POC 0 — Green

The initial frontend/runtime baseline is working.

Verified:

- Rust 1.98.1 development toolchain
- Topcoat 0.8.1
- Dockerized development environment
- Direct Topcoat HTTP serving on localhost
- Server-side rendering
- Topcoat layouts and routing
- Topcoat browser runtime
- Rust-authored client reactivity
- Topcoat asset bundling
- Plain CSS
- SCSS
- Bootstrap 5.3.8
- Locally built and self-hosted Bootstrap JavaScript
- Node isolated to the asset-builder container
- Host and container Cargo build directories isolated from each other
- `cargo check --locked` succeeds both on the host and in Docker

The application is available at:

```text
http://localhost:3000
```

The host mapping intentionally binds to `127.0.0.1` so the development application is not exposed to the LAN by default.

## Current stack

### Application

- Rust 1.98.1
- Rust edition 2024
- Topcoat 0.8.1
- Tokio

### Frontend

- Topcoat `view!`
- Topcoat reactive runtime
- Bootstrap 5.3.8
- Dart Sass
- Plain CSS

Tailwind is supported by Topcoat but is intentionally **not required** by this template.

The frontend architecture must remain compatible with standards-based CSS and must not depend on any single CSS framework.

### Infrastructure

- Docker Compose
- Rust application container
- Dedicated frontend asset-builder container
- PostgreSQL 18.6
- Docker Compose secrets

## Rust toolchain

The repository uses `rust-toolchain.toml` to pin the development toolchain:

```toml
[toolchain]
channel = "1.98.1"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

`Cargo.toml` declares:

```toml
rust-version = "1.98"
```

These serve different purposes:

- `rust-version` is the minimum supported Rust version.
- `rust-toolchain.toml` pins the exact development toolchain used by the project.

The Docker image also installs `rustfmt` and `clippy` during image construction rather than allowing the running container to mutate its Rust installation.

## Build isolation

Host Cargo and Docker Cargo intentionally use separate target directories.

Host:

```text
./target/
```

Docker:

```text
/cargo-target/
```

The Docker target directory is backed by a named Docker volume.

This prevents container-created root-owned build artifacts from interfering with local Cargo, rust-analyzer, or IDE tooling.

## Frontend asset policy

Plain CSS, SCSS, Bootstrap, and other appropriate standards-based frontend technologies are first-class options.

Current behavior:

- SCSS is compiled by a dedicated Node asset container.
- Bootstrap is installed as a build dependency.
- Bootstrap CSS is compiled locally from SCSS.
- Bootstrap JavaScript is copied into generated assets.
- Browser assets are served locally by Topcoat.
- Browser clients do not depend on a third-party Bootstrap CDN.
- Node is not part of the Rust application runtime.
- `package-lock.json` is committed for reproducible frontend builds.
- Generated frontend output is not committed.

Topcoat and the application architecture must not become dependent on Tailwind.

## Secrets and configuration

The preferred secrets mechanism is **Docker Compose secrets**, not `.env`.

Local development secrets live under:

```text
.secrets/
```

This directory is ignored by Git.

The PostgreSQL development password is created with:

```bash
mkdir -p .secrets
chmod 700 .secrets

openssl rand -hex 24 > .secrets/postgres_password
chmod 600 .secrets/postgres_password
```

The secret is exposed only to containers that explicitly receive it and is mounted inside those containers under:

```text
/run/secrets/postgres_password
```

Non-secret database configuration such as the host, port, database name, and username may use ordinary container environment variables.

The Rust application will read the database password directly from the mounted secret file instead of converting it into a password-bearing environment variable.

`.env` may be supported later as an optional convenience fallback for users of the template, but it is not the preferred configuration mechanism.

## Security philosophy

Rust memory safety does not replace application security.

The target is a security posture comparable to or stronger than a mature Laravel application.

Security controls are evaluated independently, including:

- authentication
- authorization
- session security
- CSRF protection
- origin validation
- input validation
- output escaping
- SQL injection prevention
- XSS prevention
- rich HTML sanitization
- rate limiting
- secure cookies
- security headers
- CSP
- file-upload handling
- secret management
- audit logging
- dependency auditing

Missing Topcoat functionality must be implemented using mature, well-reviewed Rust crates or proven industry patterns rather than ad-hoc replacements.

## User-input trust model

All client input is untrusted.

The intended request path is:

```text
HTTP input
    ↓
typed input struct
    ↓
field-specific normalization
    ↓
validation
    ↓
validated input
    ↓
authorization / business rules
    ↓
persistence
```

Validation and sanitization are not treated as the same operation.

Normal text should be validated and rendered through Topcoat's normal escaped output mechanisms.

Raw HTML is never trusted merely because it passed validation. If rich HTML support is introduced, it will use an allowlist sanitizer and a dedicated trusted HTML type before unescaped rendering is permitted.

Database queries must use SQLx parameter binding. User-controlled values must never be interpolated directly into SQL strings.

## Database architecture

PostgreSQL is the database target.

SQLx is the intended Rust database toolkit rather than a traditional ORM.

The design favors:

- explicit SQL
- compile-time checked queries
- strongly typed row models
- connection pooling
- embedded/versioned migrations
- compile-time model/query contracts
- restricted locations for raw SQL

Normal model-specific SQL will live with that model's persistence implementation.

Conceptually:

```text
src/db/
├── models/
│   └── contact/
│       ├── mod.rs
│       ├── model.rs
│       ├── persistence.rs
│       └── tests/
│
└── queries/
```

Responsibilities:

- `model.rs` — persisted row/data type
- `persistence.rs` — SQL and persistence behavior specific to that model
- `queries/` — complex joins, projections, reports, or queries that do not naturally belong to one model

Raw application SQL should not appear in page handlers, components, or unrelated services.

Migrations remain under:

```text
migrations/
```

because migration SQL defines the database schema itself.

## Model/schema contracts

A primary design requirement is that Rust database structs remain synchronized with the actual PostgreSQL schema.

Where practical, SQLx compile-time query checking will be used to enforce:

- column names
- Rust/PostgreSQL type compatibility
- nullability
- expected result fields

These guarantees will be deliberately tested by temporarily introducing schema/model mismatches and confirming that the build or contract test fails.

Custom reflection or procedural macros will only be introduced if SQLx does not provide sufficient guarantees.

## Forms and persisted models

Incoming request structures and persisted database structures are separate concepts.

For example:

```text
CreateContactInput
        ↓
validation
        ↓
Contact::create(...)
        ↓
Contact
```

The application should not deserialize arbitrary HTTP input directly into persisted database models.

This provides a strongly typed boundary against accidental privilege fields, mass-assignment-style mistakes, and unvalidated data reaching persistence code.

## Development

Build and start:

```bash
docker compose up --build
```

Or start existing images:

```bash
docker compose up -d
```

Check service state:

```bash
docker compose ps
```

Follow application logs:

```bash
docker compose logs -f app
```

Run the canonical container build check:

```bash
docker compose exec app cargo check --locked
```

Run the host build check:

```bash
cargo check --locked
```

Formatting:

```bash
cargo fmt --check
```

Linting:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Topcoat development reload

`topcoat::dev::script()` is currently retained because it provides Topcoat's browser-side automatic development reload/status behavior.

Under the current Docker configuration, its dynamically selected development endpoint is not exposed correctly to the host browser and can produce a refused `dev.js` request.

This does not affect application functionality.

The development endpoint will be corrected separately rather than removing the feature.

## Repository policy

Commit:

- Rust source
- frontend source
- migrations
- `Cargo.lock`
- `package-lock.json`
- `rust-toolchain.toml`
- Docker configuration
- SQLx offline metadata when introduced

Do not commit:

- `.secrets/`
- `.env`
- `target/`
- `node_modules/`
- generated frontend assets
- database data volumes
- private keys or credentials

## POC roadmap

### POC 0 — Frontend/runtime baseline

**Status: Green**

- SSR
- routing/layouts
- Bootstrap
- SCSS
- plain CSS
- assets
- client reactivity
- Dockerized development

### POC 1 — Application/data foundation

In progress:

- modular application structure
- PostgreSQL
- SQLx
- connection pool
- typed configuration
- Docker secrets
- migrations
- model/schema contracts
- first CRUD workflow

### POC 2 — Validation and forms

Planned:

- typed request models
- normalization
- validation
- safe error rendering
- form-state handling
- invalid-input testing

### POC 3 — Authentication and sessions

Planned comparison against mature Laravel/Fortify-style expectations:

- registration
- login
- logout
- secure password hashing
- session rotation
- expiration
- login throttling
- password reset
- email verification
- MFA
- credential enumeration resistance

### POC 4 — Authorization

Planned:

- policies
- RBAC
- ownership rules
- route/action authorization
- negative authorization tests

### POC 5 — Web security hardening

Planned:

- CSRF/origin protections
- CSP
- security headers
- rate limiting
- request-size limits
- secure cookies
- error handling
- audit logging

### POC 6 — Production readiness

Planned:

- tests
- dependency auditing
- container hardening
- observability
- deployment strategy
- production reverse proxy/TLS architecture

## Decision criteria

The final decision will not be based solely on performance or preference for Rust.

Topcoat/Rust must demonstrate that it can provide:

- maintainable application organization
- secure defaults or explicit secure replacements
- reasonable development velocity
- predictable framework behavior
- strong testing ergonomics
- manageable dependency risk
- clean deployment and operations

If recreating Laravel's mature safeguards creates excessive custom security or maintenance burden, that will weigh against a rewrite even if the Rust implementation performs well.
