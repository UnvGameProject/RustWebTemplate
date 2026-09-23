# Topcoat POC

A Dockerized test bed for evaluating Topcoat as a Rust frontend/full-stack framework against a Laravel/Livewire-style development and security baseline.

## Start

```bash
docker compose up --build
```

Open <http://localhost:3000>.

The host mapping is intentionally bound to `127.0.0.1`, so the development server is not exposed to the LAN by default.

## Frontend asset policy

- Plain CSS is supported directly.
- SCSS is compiled with Dart Sass 1.105.0.
- Bootstrap 5.3.8 is installed as source and compiled from SCSS.
- Bootstrap JavaScript is copied into the generated asset directory and self-hosted by Topcoat.
- Tailwind is optional and is not required by this baseline.
- Node is isolated to the development asset-builder container; it is not installed in the Rust app container.
- The first asset-container run creates `package-lock.json`; keep that lockfile in version control.

## Scope of POC 0

- Topcoat 0.8.1 pinned exactly because the framework is pre-1.0.
- Rust 1.98 baseline because `topcoat-cli 0.8.1` requires Rust 1.98 or newer.
- Server-rendered `view!` page.
- Bootstrap + SCSS + plain CSS.
- One Topcoat client-reactive interaction.
- One Bootstrap JavaScript interaction.
- Docker-only local workflow.

Database, authentication, validation, rate limiting, CSP/security headers, RBAC, and tests are intentionally added in later slices so each capability can be evaluated independently.
