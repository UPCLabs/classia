-- MFA TOTP state per user (SDS v2.2 §14.5). Applied migrations stay untouched:
-- the secret lives in dedicated encrypted columns, never in users.token.
ALTER TABLE users
    ADD COLUMN mfa_status VARCHAR(10) NOT NULL DEFAULT 'disabled',
    ADD COLUMN mfa_secret_ciphertext BYTEA,
    ADD COLUMN mfa_nonce BYTEA,
    ADD COLUMN mfa_last_step BIGINT,
    ADD COLUMN mfa_failed_attempts INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN mfa_locked_until TIMESTAMPTZ,
    ADD COLUMN mfa_enabled_at TIMESTAMPTZ;

ALTER TABLE users
    ADD CONSTRAINT users_mfa_status_ck
        CHECK (mfa_status IN ('disabled', 'pending', 'active')),
    ADD CONSTRAINT users_mfa_failed_attempts_ck
        CHECK (mfa_failed_attempts >= 0),
    ADD CONSTRAINT users_mfa_state_ck CHECK (
        (mfa_status = 'disabled'
            AND mfa_secret_ciphertext IS NULL
            AND mfa_nonce IS NULL
            AND mfa_enabled_at IS NULL)
        OR (mfa_status = 'pending'
            AND mfa_secret_ciphertext IS NOT NULL
            AND mfa_nonce IS NOT NULL
            AND mfa_enabled_at IS NULL)
        OR (mfa_status = 'active'
            AND mfa_secret_ciphertext IS NOT NULL
            AND mfa_nonce IS NOT NULL
            AND mfa_enabled_at IS NOT NULL
            AND mfa_last_step IS NOT NULL)
    );
