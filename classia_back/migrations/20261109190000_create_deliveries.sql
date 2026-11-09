CREATE TYPE delivery_status AS ENUM ('submitted', 'replaced', 'graded');

CREATE TABLE deliveries (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_url VARCHAR(255) NOT NULL,
    status delivery_status NOT NULL DEFAULT 'submitted',
    version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT deliveries_activity_student_uk UNIQUE (activity_id, student_id)
);

CREATE TABLE evaluations (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    delivery_id UUID NOT NULL UNIQUE REFERENCES deliveries(id) ON DELETE CASCADE,
    score NUMERIC(3, 1) NOT NULL CHECK (score >= 0.0 AND score <= 5.0),
    feedback TEXT,
    teacher_id UUID NOT NULL REFERENCES users(id),
    graded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE audit_result AS ENUM ('success', 'denied', 'failure');

CREATE TABLE audit_events (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    correlation_id UUID,
    actor_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    target_type VARCHAR(50) NOT NULL,
    target_id UUID,
    action VARCHAR(80) NOT NULL,
    result audit_result NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX audit_events_occurred_at_idx ON audit_events (occurred_at);
CREATE INDEX audit_events_target_idx ON audit_events (target_type, target_id);
