---
description: Generar un release Windows (.msi) del desktop y promover dev → master.
allowed-tools: Read, Bash
---

# /deploy — Release Windows

> precis se distribuye como instalador `.msi` para Windows. No hay pipeline cloud — el build se hace local.

## 1. Pre-flight

En `dev`, asegurate de que todo está limpio:

- `git status` → working tree clean.
- `pnpm lint` verde.
- `pnpm test` verde (Vitest).
- `cargo test --manifest-path src-tauri/Cargo.toml` verde.
- `pnpm test:e2e` verde (Playwright).

## 2. Bump de versión

Actualizá manualmente las **tres** versiones para que coincidan:

- `package.json` → `version`
- `src-tauri/tauri.conf.json` → `version`
- `src-tauri/Cargo.toml` → `version`

> Tauri valida que las tres coincidan en build. Si no, falla con un error confuso.

Commit: `chore(release): vX.Y.Z`.

## 3. Build release

```bash
pnpm tauri build
```

Output: `src-tauri/target/release/bundle/msi/precis_X.Y.Z_x64_en-US.msi`.

## 4. Smoke test del instalador

- Instalá el `.msi` en una máquina (o VM Windows) **sin** entorno de desarrollo.
- Validá el flujo del MVP (Compras): cargar xls de entrada → completar formulario → descargar xls de salida.
- Si algo rompe — volvé a `dev`, fix, repetí desde el paso 1.

## 5. Promover dev → master

```bash
gh pr create --base master --head dev --title "release: vX.Y.Z" --body "..."
```

Una vez aprobado y mergeado:

```bash
git switch master && git pull && git tag vX.Y.Z && git push origin vX.Y.Z
```

## 6. Distribuir

- Subí el `.msi` al canal acordado con el cliente (Drive / SharePoint / lo que sea).
- Avisá con la lista de cambios desde la versión anterior.
- Recordatorio: si el `.msi` no está firmado, Windows SmartScreen va a alertar al cliente — comentarlo de antemano.
