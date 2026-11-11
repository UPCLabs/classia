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

## MFA (UC-005)

Set `MFA_ENCRYPTION_KEY` to a separately generated 32-byte AES key encoded as
64 hexadecimal characters. Docker Compose requires this variable; direct backend
execution without it keeps password-only accounts working but refuses MFA setup
and verification. An invalid configured key prevents router initialization.
Never commit the real key or reuse `JWT_SECRET`.

Generate a key locally, then store it in the private deployment environment:

```bash
python3 -c 'import secrets; print(secrets.token_hex(32))'
```

Keep this key stable across restarts and back it up separately from PostgreSQL.
Changing or losing it makes existing MFA secrets unreadable. There is no automatic
key rotation or account recovery in UC-005.

The backend applies a new additive migration at startup. Existing users begin
with MFA disabled; no database reset is needed. Previously applied migrations
retain their `main` checksums. If a database ran a different historical migration,
investigate its checksum mismatch rather than deleting data or editing SQLx history.

### HTTP flow

- `POST /api/auth/login` with `{email,password}`: disabled or pending MFA returns
  `204` and the normal session. Active MFA returns `200 {mfa_required:true}`,
  clears the access cookie and sets a five-minute partial cookie.
- `POST /api/auth/setup-mfa` with a full session: returns `{secret,url}` with
  `Cache-Control: no-store`. The secret is pending until a valid code is submitted.
  Repeating setup replaces the pending secret, but cannot clear an active lock.
- `POST /api/auth/challenge-mfa` with `{code}`: a full session activates the pending
  factor; a partial session completes login. Only successful login verification
  issues a new access cookie. Partial sessions cannot access business routes.

Codes use RFC 6238 (SHA-1, six digits, 30 seconds, ±1 step). Accepted steps cannot
be reused. Five invalid codes lock verification for five minutes; `429 MFA_LOCKED`
includes `Retry-After` in seconds. User-row locks serialize concurrent challenges.
Secrets are encrypted with AES-256-GCM and bound to the user UUID.

`mfa.setup_started`, `mfa.activated` and `mfa.challenge` are recorded in
`audit_events` without codes, secrets or otpauth URIs. Successful state changes
and their audit event commit atomically. Denials are recorded after rollback;
failed-code counters persist before their failure event is recorded.

Tokens without the new `purpose` claim are rejected, so existing sessions must
log in again after this update. Cookies remain `HttpOnly`, `Secure`, `SameSite=Lax`;
the partial cookie is restricted to `/api/auth`. Use HTTPS outside loopback.

Administrative reset, session-version revocation and UC-006 are **not implemented
by this change**, even if the existing frontend exposes a reset control.

### Validation

Run the complete suite against an isolated PostgreSQL server using the test command
above. The suite covers activation, encrypted storage, secret replacement, replay,
partial-session isolation and expiry, lock recovery, concurrent verification,
audit rollback and upgrading a database containing existing users.

These HTTP tests do not replace a real browser/authenticator QR check.
