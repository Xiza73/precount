---
description: Revisión de código del diff actual contra `dev` (o working tree si no hay diff). Reporta hallazgos por capa categorizados como CRITICAL / WARNING / SUGGESTION.
allowed-tools: Read, Glob, Grep, Bash
---

# /revision — Revisión de código

Hacé una revisión técnica del **diff actual contra `dev`** (o de los archivos modificados en el working tree si no hay diff todavía). Andá capa por capa.

## 1. Frontend (`src/`)

- ¿Componentes React respetan React 19 (sin `useMemo`/`useCallback` innecesarios)?
- ¿Tipos TS estrictos, sin `any` injustificados?
- ¿Tests Vitest cubren la lógica nueva?
- ¿La feature está dentro de `src/modules/<modulo>` y reutiliza primitivas de `src/lib/`?

## 2. Backend Tauri (`src-tauri/`)

- ¿Cada Tauri command nuevo tiene su test `#[cfg(test)]`?
- ¿Errores manejados con `Result<T, AppError>` propio, **sin** `unwrap()`/`expect()` en código de producción?
- ¿Archivos de entrada se procesan en memoria (no se escriben a disco)?
- ¿`clippy` sin warnings?

## 3. Arquitectura

- ¿La feature respeta la estructura modular (`src/modules/<x>` + `src-tauri/src/commands/<x>.rs`)?
- ¿Se reutilizaron helpers existentes (validación de RUC, parseo de fechas) en vez de duplicarlos?

## 4. SISCONT compliance

- Si se tocó el formato de salida XLS: ¿el layout sigue siendo el que validó el cliente?
- Si es un módulo nuevo: ¿está el formato documentado en el código o en el módulo correspondiente?

## Output esperado

Formato:

```
## CRITICAL
- <file>:<line> — <qué está mal> → <cómo arreglarlo>

## WARNING
- <file>:<line> — <qué está mal> → <cómo arreglarlo>

## SUGGESTION
- <mejora opcional>

## OK
- <qué se revisó sin hallazgos>
```

Si todo está limpio, decilo explícitamente y enumerá qué se cubrió. No inventes problemas.
