# DEV-16 — Cliente de API de entregas, evaluaciones y MFA

Documento de apoyo para revisar la rama `mp/dev-16-api-client`.

## Qué hace este código

Es la **capa de acceso a la API** del frontend para las funcionalidades del Ciclo 2:

- Subir y reemplazar entregas (UC-015 y UC-016).
- Consultar el estado de una entrega y las entregas recibidas (UC-017).
- Descargar el archivo de una entrega y calificarla (UC-018).
- Configurar, verificar y restablecer MFA (UC-005 y UC-006).

No tiene interfaz propia: son funciones y tipos que consumen las pantallas de las ramas siguientes. Toda la comunicación pasa por `src/api/httpClient.ts`, el único cliente Axios del proyecto (`baseURL: '/api'`, cookies habilitadas).

## Archivos

| Archivo | Contenido |
|---|---|
| `src/types/api.ts` | Tipos `Entrega`, `Evaluacion`, `EntregasRecibidas`, `CalificacionInput`, `MfaSetup`, etc. |
| `src/api/deliveries.ts` | Llamadas HTTP de entregas + validación local del archivo. |
| `src/api/mfa.ts` | Llamadas HTTP de MFA. |
| `tests/api-deliveries.test.ts` | 12 pruebas del cliente de entregas. |
| `tests/api-mfa.test.ts` | 5 pruebas del cliente de MFA. |

## Tipos principales

```ts
type DeliveryStatus = 'submitted' | 'replaced' | 'graded'

type Evaluacion = {
  score: number
  feedback: string | null
  graded_at: string
  updated_at: string
}

type Entrega = {
  id: string
  activity_id: string
  student: { id: string; name: string; email: string }
  status: DeliveryStatus
  version: number
  submitted_at: string
  updated_at: string
  evaluation: Evaluacion | null
}
```

`EntregaPropia` es `{ delivery: Entrega | null }` (la respuesta de "mi entrega") y `EntregasRecibidas` es `{ items: Entrega[]; total: number }`.

## Endpoints y funciones

Contrato: Issue #46 (DEV-10).

| Función | Método y ruta | Cuerpo / respuesta |
|---|---|---|
| `submitDelivery(activityId, file)` | `POST /activities/{id}/deliveries` | `multipart/form-data` con `file` → `Entrega` |
| `replaceDelivery(deliveryId, file, expectedVersion)` | `PUT /deliveries/{id}` | `multipart` con `file` y `expected_version` → `Entrega` |
| `getMyDelivery(activityId)` | `GET /activities/{id}/deliveries/me` | `{ delivery: Entrega \| null }` → `Entrega \| null` |
| `listReceivedDeliveries(activityId)` | `GET /activities/{id}/deliveries` | `{ items, total }` |
| `downloadDeliveryFile(deliveryId)` | `GET /deliveries/{id}/file` | `Blob` (respuesta binaria) |
| `gradeDelivery(deliveryId, input)` | `PUT /deliveries/{id}/evaluation` | `{ score, feedback?, expected_version }` → `Entrega` |

Funciones de MFA (contrato provisional, ver más abajo):

| Función | Método y ruta | Cuerpo / respuesta |
|---|---|---|
| `setupMfa()` | `POST /auth/setup-mfa` | → `{ secret, url }` |
| `challengeMfa(code)` | `POST /auth/challenge-mfa` | `{ code }` → `{ verified: boolean }` |
| `resetMfa(userId, reason)` | `POST /users/{id}/reset-mfa` | `{ reason }` |

## Cómo funciona por dentro

### 1. Multipart sin fijar `Content-Type`

```ts
function fileFormData(file: File): FormData {
  const form = new FormData()
  form.append('file', file)
  return form
}
```

Al enviar un `FormData`, Axios y el navegador generan el `boundary` automáticamente. **No se debe** poner `Content-Type: multipart/form-data` a mano: si se fija, el boundary falta y el servidor no puede separar las partes.

En el reemplazo se agrega la versión que vio el estudiante:

```ts
form.append('expected_version', String(expectedVersion))
```

Esto implementa la concurrencia optimista de DEC-04: si otra persona reemplazó el archivo, el servidor responde `409 DELIVERY_VERSION_CONFLICT`.

### 2. Validación local del archivo

`validateDeliveryFile(file)` replica las reglas del contrato antes de gastar una petición:

- Extensión `.docx` → si no, `"El archivo debe estar en formato Docx"`.
- Tamaño máximo 20 MB (`MAX_DELIVERY_BYTES`) → si lo supera, `"El archivo supera el tamaño máximo de 20 MB"`.
- Devuelve `null` cuando el archivo es válido.

El servidor vuelve a validar; esto solo mejora la experiencia.

### 3. Errores

El interceptor de `httpClient` convierte las respuestas de error de la API en `{ code, message }`. El cliente **no los envuelve**: la promesa se rechaza con ese objeto, de modo que las pantallas pueden decidir según el código:

```ts
try {
  await replaceDelivery(id, file, version)
} catch (error) {
  if ((error as ApiError).code === 'DELIVERY_VERSION_CONFLICT') {
    // recargar la entrega y avisar
  }
}
```

Códigos relevantes: `DELIVERY_VERSION_CONFLICT` (409), `DUE_DATE_PASSED` (400), `INVALID_FILE_TYPE` (400), `FILE_TOO_LARGE` (400), `DELIVERY_ALREADY_EXISTS` (409).

### 4. Descarga

`downloadDeliveryFile` pide `responseType: 'blob'`. La pantalla convierte el `Blob` en un enlace temporal y lo descarga; el nombre original nunca se expone (DEC-03).

### 5. MFA (provisional)

El contrato de MFA aún no está publicado. Estas rutas siguen el backend en progreso (`sm/add-mfa`) y están aisladas en `src/api/mfa.ts`: cuando David publique el contrato, **solo ese archivo cambia**.

## Pruebas

```bash
cd classia_front
npx vitest run tests/api-deliveries.test.ts tests/api-mfa.test.ts
```

Cubren:

- El `FormData` contiene el archivo correcto.
- El reemplazo envía `expected_version`.
- `getMyDelivery` devuelve `null` cuando no hay entrega.
- La lista conserva `items` y `total`.
- La descarga pide `responseType: 'blob'`.
- La calificación envía nota, retroalimentación y versión.
- Los códigos 400 y 409 se propagan sin perder `code`.
- Validación de extensión y tamaño.
- MFA: QR/secreto, código válido e inválido, y motivo del restablecimiento.

## Ejemplo de uso en una pantalla

```ts
const entrega = await getMyDelivery(activityId)
if (!entrega) {
  const nueva = await submitDelivery(activityId, file)
}
```

## Limitaciones actuales

- Las rutas de entregas todavía **no existen en el backend de `main`**: las pantallas muestran 404 hasta que entren DEV-12, DEV-13 y DEV-14.
- Las pruebas simulan HTTP (`vi.mock` de `httpClient`); no prueban el backend real.
- Admin y SuperAdmin no tienen acceso a entregas (decisión A-01 del contrato).
