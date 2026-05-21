---
name: security-review
description: Audita seguridad de Tauri commands, parsing de archivos del usuario, capability config, CSP y vectores de path-traversal. Disparar al revisar PRs que tocan I/O, parsing xls/csv, o configuración Tauri, y antes de cada release.
---

# Security review — precis

precis es local-only, pero **procesa archivos no confiables del usuario**. El threat model gira alrededor de:

1. **Archivos maliciosos** subidos por el usuario (xls/csv/xml, zip-bombs en xlsx).
2. **Path traversal** en Tauri commands que tocan filesystem.
3. **Capabilities Tauri** demasiado abiertas en `tauri.conf.json`.

## Checklist obligatoria

### Tauri commands (`src-tauri/src/commands/`)

- [ ] Ningún command acepta paths absolutos del frontend sin validar contra un allowlist.
- [ ] No se escribe a disco contenido del usuario (regla del proyecto). Excepción justificada → `tempfile` + cleanup garantizado en `Drop`.
- [ ] Argumentos del command tipados estrictamente (no `serde_json::Value` libre).
- [ ] Errores no exponen paths del sistema ni stack traces al frontend.

### Parsing de archivos

- [ ] Tamaño máximo del archivo de entrada validado **antes** de parsear (mitigación de zip-bombs en xlsx).
- [ ] `calamine` cargado con límites razonables — no leer hojas de millones de filas a memoria.
- [ ] CSV con `csv` crate y límite de campos por fila.

### Tauri config (`src-tauri/tauri.conf.json`)

- [ ] `app.security.csp` definido y restrictivo (sin `'unsafe-inline'` en `script-src`).
- [ ] Capabilities mínimas: solo los plugins efectivamente usados.
- [ ] `dangerousDisableAssetCspModification: false`.

### Frontend

- [ ] Sin `dangerouslySetInnerHTML` con contenido derivado del input del usuario.
- [ ] Nombres de archivo sanitizados antes de renderizar en UI.

## Output

```
## CRITICAL — bloquea release
- <file>:<line> — <descripción> → <fix>

## HIGH — fix en el sprint en curso
- ...

## MEDIUM — backlog
- ...

## CLEAN
- <áreas revisadas sin hallazgos>
```

Si todo está OK, decilo explícitamente y enumerá qué se cubrió. No inventes vulnerabilidades para llenar el reporte.
