-- Classia — Datos de prueba de integración del Ciclo 2
-- Plan de Pruebas v2.1, sección 2.1.3 (Ambiente).
--
-- Ejecutar inmediatamente después de datos_prueba_integracion_ciclo1.sql, que
-- borra y recrea los usuarios @classia.test, los cursos IS-001, BD-001 y
-- RD-001 y las actividades Taller 1 y Taller 2 de IS-001.
--
-- Agrega a IS-001:
--   - Taller 3: plazo vencido (publicada, límite 31/10/2026 23:59 hora de
--     Bogotá) con una entrega de estudiante1.
--   - Taller 4: borrador vencido (borrador, mismo límite).
-- La entrega sembrada es solo un registro: no tiene archivo en el volumen y
-- no debe descargarse. Es idempotente junto con el script del Ciclo 1.
--
-- Uso con el stack de Docker Compose (desde la raíz del repositorio):
--   docker compose exec -T postgres sh -c 'psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB"' \
--     < classia_back/testdata/datos_prueba_integracion_ciclo1.sql
--   docker compose exec -T postgres sh -c 'psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB"' \
--     < classia_back/testdata/datos_prueba_integracion_ciclo2.sql

BEGIN;

INSERT INTO activities
    (id, course_id, title, description, due_at, status, published_at)
SELECT seed.id::uuid, course.id, seed.title, seed.description,
       seed.due_at::timestamptz, seed.status::activity_status,
       seed.published_at::timestamptz
FROM (VALUES
    (
        '019a4b10-0000-7000-8000-000000000003',
        'IS-001',
        'Taller 3: plazo vencido',
        'Entregar el informe de requisitos del curso.',
        '2026-11-01T04:59:00Z',
        'published',
        '2026-10-20T22:00:00Z'
    ),
    (
        '019a4b10-0000-7000-8000-000000000004',
        'IS-001',
        'Taller 4: borrador vencido',
        'Revisar el modelo de datos del curso.',
        '2026-11-01T04:59:00Z',
        'draft',
        NULL
    )
) AS seed(id, course_code, title, description, due_at, status, published_at)
JOIN courses course ON course.code = seed.course_code;

INSERT INTO deliveries (id, activity_id, student_id, file_url, submitted_at, updated_at)
SELECT '019a4b20-0000-7000-8000-000000000001'::uuid,
       '019a4b10-0000-7000-8000-000000000003'::uuid,
       student.id,
       '019a4b20-0000-7000-8000-000000000001.docx',
       '2026-10-30T15:00:00Z', '2026-10-30T15:00:00Z'
FROM users student
WHERE student.email = 'estudiante1.cpi@classia.test';

COMMIT;

-- Verificación rápida del estado inicial esperado:
--   IS-001 con 4 actividades (2 publicadas y 2 borradores) y 1 entrega.
SELECT a.title, a.status, a.due_at AT TIME ZONE 'America/Bogota' AS limite_bogota,
       COUNT(d.id) AS entregas
FROM activities a
JOIN courses c ON c.id = a.course_id
LEFT JOIN deliveries d ON d.activity_id = a.id
WHERE c.code = 'IS-001'
GROUP BY a.title, a.status, a.due_at
ORDER BY a.due_at, a.title;
