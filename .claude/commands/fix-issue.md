---
description: Workflow guiado para resolver un bug — reproducir con un test que falle, encontrar root cause, fixear, y commitear.
argument-hint: <descripción del bug o número de issue>
allowed-tools: Read, Edit, Write, Glob, Grep, Bash
---

# /fix-issue — Workflow de bug fix

Issue/bug: $ARGUMENTS

Seguí este flujo **en orden**. No saltes pasos.

## 1. Entender el síntoma

- Leé la descripción del issue/bug.
- Si referencia código, abrilo y mapeá el flujo desde el evento del usuario hasta el síntoma.

## 2. Reproducir con un test que falle

- Escribí un test que **rojee** demostrando el bug:
  - UI / lógica TS → Vitest.
  - Tauri command / lógica Rust → `cargo test`.
  - Flujo end-to-end → Playwright.
- Confirmá que el test falla por la razón correcta (no por un error de setup).

## 3. Root cause

- No parches el síntoma. Encontrá *por qué* pasa.
- Si dudás entre dos causas posibles, decilo y pedí input antes de tocar código.

## 4. Fix

- Aplicá el cambio **mínimo** que verde el test.
- Corré el resto de la suite para asegurar que no rompiste otra cosa:

```bash
pnpm test && cargo test --manifest-path src-tauri/Cargo.toml
```

## 5. Commit

- Mensaje: `fix(<scope>): <qué se arregló>`.
- Body: una línea con el **root cause**.
- NO incluyas attribution AI.

## 6. Memoria

- Si fue un bug no-obvio, llamá `mem_save` con `type: bugfix` y el root cause registrado, para que la próxima vez el contexto esté disponible.
