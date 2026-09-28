-- Classia — Datos de prueba de integración del Ciclo 1
-- Plan de Pruebas de Integración v1.1, sección 2.1.3 (Ambiente).
--
-- Deja la base de datos en el estado inicial de los casos CPI-001 a CPI-019.
-- Ejecutar antes de cada ronda de pruebas y antes de repetir un caso marcado
-- con "Recargar los datos de prueba". Es idempotente: se puede ejecutar varias
-- veces seguidas.
--
-- Solo borra datos de prueba: usuarios con correo @classia.test, sus cursos e
-- inscripciones, y los cursos con los códigos usados en los casos. No toca el
-- Superadministrador de arranque ni otros datos.
--
-- Uso con el stack de Docker Compose (desde la raíz del repositorio):
--   docker compose exec -T postgres sh -c 'psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB"' \
--     < classia_back/testdata/datos_prueba_integracion_ciclo1.sql
--
-- Contraseña de todos los usuarios: Classia.Prueba.2026
-- (hash Argon2id generado con el crate argon2 0.6.0 del backend).

BEGIN;

DELETE FROM course_enrollments
WHERE student_id IN (SELECT id FROM users WHERE email LIKE '%@classia.test');

DELETE FROM courses
WHERE code IN ('IS-001', 'BD-001', 'RD-001', 'AS-001')
   OR teacher_id IN (SELECT id FROM users WHERE email LIKE '%@classia.test');

DELETE FROM users
WHERE email LIKE '%@classia.test';

INSERT INTO users (name, email, password, role, status, token)
SELECT name, email,
       '$argon2id$v=19$m=19456,t=2,p=1$lKsg5oVj8J9t1hfnXIOjFw$SoUsiCzW/A+nWMUj9eoV4nySi/So6Q0PNc+yLIvvHwg',
       role::user_role, 'active', ''
FROM (VALUES
    ('Administrador CPI',      'admin.cpi@classia.test',       'admin'),
    ('Docente Uno CPI',        'docente1.cpi@classia.test',    'teacher'),
    ('Docente Dos CPI',        'docente2.cpi@classia.test',    'teacher'),
    ('Estudiante Uno CPI',     'estudiante1.cpi@classia.test', 'student'),
    ('Estudiante Dos CPI',     'estudiante2.cpi@classia.test', 'student'),
    ('Estudiante Tres CPI',    'estudiante3.cpi@classia.test', 'student'),
    ('Cambio Contraseña CPI',  'cambio.cpi@classia.test',      'student')
) AS seed(name, email, role);

INSERT INTO courses (name, code, teacher_id, status, capacity)
SELECT seed.name, seed.code, teacher.id, 'active', seed.capacity
FROM (VALUES
    ('Ingeniería de Software', 'IS-001', 'docente1.cpi@classia.test', 30),
    ('Bases de Datos',         'BD-001', 'docente1.cpi@classia.test', 1),
    ('Redes',                  'RD-001', 'docente2.cpi@classia.test', 20)
) AS seed(name, code, teacher_email, capacity)
JOIN users teacher ON teacher.email = seed.teacher_email;

INSERT INTO course_enrollments (course_id, student_id)
SELECT course.id, student.id
FROM (VALUES
    ('IS-001', 'estudiante1.cpi@classia.test'),
    ('BD-001', 'estudiante2.cpi@classia.test'),
    ('RD-001', 'estudiante2.cpi@classia.test')
) AS seed(code, student_email)
JOIN courses course ON course.code = seed.code
JOIN users student ON student.email = seed.student_email;

COMMIT;

-- Verificación rápida del estado inicial esperado:
--   7 usuarios @classia.test, 3 cursos y 3 inscripciones.
SELECT c.code, c.name, t.email AS docente, c.capacity,
       COUNT(e.student_id) AS inscritos
FROM courses c
JOIN users t ON t.id = c.teacher_id
LEFT JOIN course_enrollments e ON e.course_id = c.id
WHERE c.code IN ('IS-001', 'BD-001', 'RD-001')
GROUP BY c.code, c.name, t.email, c.capacity
ORDER BY c.code;
