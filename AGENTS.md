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

## Current project context: Cycle 2, Initiation and Strategy

The team package `paquete-equipo/Guia_Quien_Hace_Que_Iniciacion_Estrategia_C2.md`
and the Cycle 2 kickoff minutes are the references for current assignments and
deadlines. Do not treat this summary as a replacement for those documents.

- The combined Initiation and Strategy submission is due 21/10/2026. Its
  deliverables are updates based on Cycle 1 documents; preserve the originals
  and label unapproved work as a draft.
- Cycle 2 scope: create, publish and consult activities; register submissions
  and consult their status. Reuse the Cycle 1 authentication, users, courses
  and persistence.
- Cycle 2 roles: Fabian Andres Baron Barbosa — Team Leader; Mario Pinto
  Linares — Planning Manager; Juanita Campos Cardenas — Quality/Process
  Manager; Esteban Guzman Paez — Development Manager; David Santiago Mendoza
  Yasno — Architecture Manager.
- Fabian integrates the Initiation Record v2.0. Mario supplies the team's
  availability for that record and integrates Strategy v2.0. Esteban
  integrates the Risk Management Plan v2.0; David integrates the Configuration
  Plan v2.0.
- Mario's planning responsibilities include consolidating the five individual
  estimates, comparing the Cycle 2 estimate with the historical 1,000 LOC /
  120-hour baseline and Cycle 1 actuals (8,313 LOC; 107.29 LOC/hour),
  maintaining the assignment tracker with actual time, and updating the
  document master at closeout.
- Mario's individual estimate is EST-02: "Create activity", LOC plus Cycle 1
  historical productivity. The team-package deadline is 18/10/2026 at 19:00;
  individual Moodle upload is on 19/10/2026, with its cutoff to be confirmed
  in Moodle.
- The Cycle 1 availability table may be reused as historical availability
  when requested, but do not describe it as newly confirmed for Cycle 2.
- The Cycle 1 Risk Plan v1.1 assigns RGP-3 and RGP-4 follow-up to Mario.
  Retain those assignments as the current source unless Esteban's Cycle 2
  risk assignment supersedes them; do not claim follow-up actions occurred
  without evidence.
- The package identifies internal checkpoints: remaining individual
  estimates 18/10 at 19:00; Mario's strategy consolidation/comparison and
  risk updates 19/10 at 19:00; Strategy integration 19/10 at 22:00;
  inspection 20/10 at 12:00, corrections at 17:00, final review at 19:00,
  and closeout at 23:00.
