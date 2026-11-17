# Local demonstration

Open this collection in ApiArk and select **Local demo**. Requests and variables
can also be recreated in Postman. This is the disposable local stack, not the VPS.
The course ID belongs to the seeded local database; recreate/select a course and
update `COURSE_ID` if that database is replaced.

Local test accounts only:

- Teacher: `docente@classia.local`, password `Demo-Pass-2026!`.
- Student: `estudiante@classia.local`, password `Demo-Pass-2026!`.

Never use these passwords in production. Do not save session cookies or TOTP
secrets in the shared collection. Cookie sending/storage is enabled for the
collection, but cookie persistence is disabled. If the client refuses `Secure`
cookies over local HTTP, use local HTTPS or explicitly supply the returned cookie
only for this local test; do not weaken production cookie attributes.

## Activities: teacher → student

1. Set the existing `auth/login` request body to the teacher credentials and send
   it. Expect `204`; `auth/me` should identify the teacher (`200`).
2. Send `activities/list` to show the course's current activities.
3. Send `activities/create`. Expect `201`; copy its response `id` to `ACTIVITY_ID`.
4. Send `activities/update` to change the due date. Expect `200`.
5. Log out, then log in as the student. List activities: the new draft must not
   appear. Set `ACT_PUBLISHED_ID` temporarily to the draft ID and send
   `activities/get`: expect `404` (the draft is hidden from the student).
6. Log out and log in as the teacher. Send `activities/publish`: expect `200`.
   Copy `ACTIVITY_ID` to `ACT_PUBLISHED_ID`.
7. Log out and log in as the student. Listing must include the published activity;
   `activities/get` must return it with `200`.
8. As the student, attempt `activities/create`: expect `403`.

Use a new title for each rehearsal. Future example deadlines are intentional.
The collection does not automatically capture response IDs: update the variables
explicitly after creating/publishing an activity.

## UC-005: optional setup, mandatory second factor once active

Use a disposable local account if you do not want MFA enabled on a demo account.
UC-006 reset is not part of this change.

1. Log in without active MFA (`204`). Send `auth/setup_mfa` (`200`) and scan its
   `url` as a QR through the frontend, or enter its Base32 `secret` manually in an
   authenticator. Neither value should be shared or committed.
2. Put the authenticator's current six-digit code in `MFA_CODE` and send
   `auth/challenge_mfa`. Expect `200 {verified:true}`: MFA is now active.
3. Log out and log in again. Expect `200 {mfa_required:true}` and only the partial
   session. `auth/me` must return `401` before the second factor.
4. Wait for the authenticator to show a **new code** (the activation code cannot be
   reused), update `MFA_CODE` and send `auth/challenge_mfa`. Expect `200`; `auth/me`
   must now work.

The backend HTTP flow is automated in the Rust suite. ApiArk's own cookie handling
and scanning a displayed QR with a real authenticator still require a manual run.
