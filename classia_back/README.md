# Backend Classia

## Tests

Unit tests do not require external services:

```bash
cargo test --locked --lib
```

The integration tests use `#[sqlx::test]` and require a PostgreSQL server whose
user can create temporary databases. Point `DATABASE_URL` to an empty test
database, then run:

```bash
DATABASE_URL=postgres://user:password@localhost:5432/classia_tests \
  cargo test --locked --all-targets
```

## TODO
- We need to indentify the schema for the database
