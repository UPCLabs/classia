# Backend pendiente para creación y edición de cursos

La fase frontend consume los endpoints y datos definidos en
`alcance-frontend-ciclo-1.md`. En la rama `JC/Control-cursos` revisada, el
backend todavía expone un contrato anterior; por ello las llamadas nuevas no
funcionarán end-to-end hasta alinear e implementar estas rutas.

## Contrato que espera el frontend

- `GET /api/courses` -> `{ "items": [Curso] }`
- `GET /api/courses/{courseId}` -> `Curso`
- `POST /api/courses` -> `201` y `Curso`
- `PATCH /api/courses/{courseId}` -> `Curso actualizado`
- `GET /api/users?role=Teacher&status=active` -> `{ "items": [Usuario] }`

Los campos del curso son `name`, `code`, `teacher_id`, `status` y `capacity`.
El objeto de respuesta incluye el docente anidado (`teacher.id`, `teacher.name`,
`teacher.email`) y `enrolled_count`. No debe exponer contraseñas.

## Estado actual del backend de cursos

La API existente registra estas rutas:

- `GET /api/courses/ById/{id_course}`
- `GET /api/courses/getCourseByCode/{course_code}`
- `GET /api/courses/getTeachersCourse/{teacher_id}`
- `POST /api/courses/create`

La creación actual requiere además `password`, usa `quantity` en lugar de
`capacity` y responde con `teacher_id` plano. No existen la lista general ni
la ruta `PATCH` de edición. La consulta por docente también devuelve un solo
curso en vez de una lista.

El backend de usuarios actual no ofrece el listado de docentes que necesita el
selector del formulario.

## Autorización necesaria

- Restringir la creación y edición a `Teacher`, `Admin` y `SuperAdmin`, según la
  política acordada.
- El servidor debe comprobar que un docente solo cree/asigne cursos propios y
  solo edite cursos cuyo `teacher_id` coincide con su identidad.
- Validar el rol del docente asignado, estado/cupo/código y que el PATCH tenga
  al menos un campo modificable.
- No confiar en las restricciones del router frontend como mecanismo de
  autorización.
- Añadir pruebas de integración para permisos, docente no propietario,
  edición vacía, curso inexistente y validación de campos.
