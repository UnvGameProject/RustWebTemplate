#!/usr/bin/env sh
set -eu

fail()
{
    printf '%s\n' "TEST SAFETY GATE: $*" >&2
    exit 64
}

[ "${APP_ENV:-}" = "testing" ] \
    || fail "APP_ENV must be testing"

[ "${DB_HOST:-}" = "postgres-test" ] \
    || fail "DB_HOST must be postgres-test"

[ "${DB_PORT:-}" = "5432" ] \
    || fail "DB_PORT must be 5432"

[ "${DB_NAME:-}" = "postgres" ] \
    || fail "DB_NAME must be postgres"

[ "${DB_USER:-}" = "topcoat_test_admin" ] \
    || fail "DB_USER must be topcoat_test_admin"

[ "${DB_PASSWORD_FILE:-}" = "/run/secrets/postgres_test_password" ] \
    || fail "unexpected database secret path"

[ -r "$DB_PASSWORD_FILE" ] \
    || fail "test database secret is not readable"

for forbidden in \
    /run/secrets/postgres_bootstrap_password \
    /run/secrets/postgres_migrator_password \
    /run/secrets/postgres_app_password
do
    [ ! -e "$forbidden" ] \
        || fail "non-test database secret is mounted: $forbidden"
done

for forbidden in \
    /app/.secrets/postgres_bootstrap_password \
    /app/.secrets/postgres_migrator_password \
    /app/.secrets/postgres_app_password
do
    [ ! -e "$forbidden" ] \
        || fail "host database secret is visible: $forbidden"
done

password="$(cat "$DB_PASSWORD_FILE")"

export DATABASE_URL="postgres://${DB_USER}:${password}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

exec cargo test --locked --all-targets "$@"
