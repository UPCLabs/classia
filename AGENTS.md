# Agent Instructions

Classia is a prototype virtual-education platform. This file holds stable
repository conventions only; product scope and the API contract live in the
team's project documentation.

## Repository layout

- `classia_back/` — Rust backend: Axum, SQLx, PostgreSQL, JWT in an
  `HttpOnly` cookie.
  - `src/<domain>/` — one module per domain (`auth`, `users`, `courses`, ...)
    split into `api.rs` (routes/handlers), `dto.rs`, `models.rs`,
    `repository.rs` (SQL), `service.rs` (business rules), `error.rs` and
    `tests.rs`. Handlers stay thin; rules go in the service; SQL only in the
    repository. New domains get their own module and are nested in `lib.rs`.
  - `migrations/` — SQLx migrations, applied automatically at startup.
  - `tests/` — integration tests using `#[sqlx::test]`.
- `classia_front/` — React, TypeScript, Vite, Tailwind, TanStack Query.
  - `src/api/httpClient.ts` — the only HTTP client; base URL is `/api`.
  - `src/views/`, `src/auth/`, `src/layouts/`; tests in `tests/` (Vitest).
- `classia_api_test/` — APIArk request collection for manual API checks.
- `docker-compose.yml` — PostgreSQL, backend and frontend. Nginx in the
  frontend container proxies `/api/` to the backend.

## Commands

Backend (`classia_back/`):

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --lib
```

`cargo test --locked --all-targets` also runs the `#[sqlx::test]` integration
tests. They create temporary databases, so point `DATABASE_URL` at a
disposable PostgreSQL instance whose user can create databases, never at a
shared or production database.

Frontend (`classia_front/`):

```bash
npm ci
npm run lint
npx vitest run
npm run build
```

Full stack (repository root):

```bash
cp .env.example .env   # then set real local values
docker compose up --build
```

The app is served at `http://127.0.0.1:8080`.

## Rules

- Never commit `.env` files, credentials or tokens. Add new variables to
  `.env.example` with placeholder values.
- Change the schema only through a new migration file. Never edit a migration
  that is already on `main`.
- Never return password hashes or tokens in API responses.
- When a route, request or response changes, update the frontend callers and
  both test suites in the same pull request.
- Frontend tests mock HTTP; they do not prove the backend route exists.
  Verify integration against the running stack.
- Run the backend and frontend checks above before opening a pull request.

## Git workflow

- Branch names start with the author's initials: `<initials>/<short-description>`
  (for example, `sm/add-agents-md`).
- Do not push directly to `main`; open a pull request and merge after review.
- Keep each pull request focused on one change.
