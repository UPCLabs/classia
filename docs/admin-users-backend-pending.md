# Backend pendiente para administración de usuarios

El frontend de administración de usuarios consume los contratos descritos en
`alcance-frontend-ciclo-1.md`. Las siguientes rutas todavía no están registradas
en el backend y deben implementarse antes de que el listado, edición y estado
funcionen end-to-end.

## Rutas que faltan

### Listar y filtrar usuarios

```http
GET /api/users?q={texto}&role={rol}&status={estado}
```

Debe estar restringida a `SuperAdmin` y `Admin`, admitir filtros opcionales y
responder con `{ "items": [Usuario] }`.

### Consultar un usuario

```http
GET /api/users/{userId}
```

Debe responder con el usuario seleccionado, sin incluir campos sensibles como
el hash de contraseña.

### Editar un usuario

```http
PATCH /api/users/{userId}
```

Debe aceptar opcionalmente `name`, `email` y `role`, exigir al menos un campo,
validar sus valores y responder con el usuario actualizado.

### Activar o desactivar un usuario

```http
PATCH /api/users/{userId}/status
```

Debe aceptar `{ "status": "active" | "inactive" }`, validar permisos y
responder con el usuario actualizado.

## Diferencia en la creación existente

El backend actual crea usuarios mediante:

```http
POST /api/users/create
```

Requiere una sesión `SuperAdmin` o `Admin`; recibe `name`, `email`, `password`
y `role`. Responde `200` con `{ "message": "User created" }`. El frontend usa
esta ruta existente.

El documento de alcance describe en cambio `POST /api/users`, con respuesta
`201` y el usuario creado. Si se desea adoptar ese contrato REST, hay que
actualizar el backend y luego cambiar el frontend para invalidar/refrescar el
listado con la respuesta de creación.

## Criterios de seguridad y respuesta

- Los endpoints de listado, detalle, edición y estado deben autorizar
  `SuperAdmin` y `Admin` en el servidor; la protección de rutas del frontend no
  reemplaza esta comprobación.
- Las respuestas de error deben conservar `{ "code": "...", "message": "..." }`.
- Las respuestas de usuario no deben exponer contraseñas ni hashes.
- Las pruebas de integración deben cubrir roles no autorizados, filtros,
  edición vacía, correo duplicado, estados inválidos y usuario inexistente.
