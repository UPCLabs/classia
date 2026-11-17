-- Session revocation for the MFA reset (SDS v2.2 §14.5, UC-006). Every JWT
-- carries the version; incrementing it invalidates earlier sessions.
ALTER TABLE users
    ADD COLUMN token_version BIGINT NOT NULL DEFAULT 0;
