CREATE TYPE activity_status AS ENUM ('draft', 'published');

CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    title VARCHAR(150) NOT NULL,
    description TEXT NOT NULL,
    due_at TIMESTAMPTZ NOT NULL,
    status activity_status NOT NULL DEFAULT 'draft',
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK ((status = 'draft') = (published_at IS NULL))
);

CREATE INDEX activities_course_status_due_idx
    ON activities (course_id, status, due_at);
