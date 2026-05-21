---
name: deploy
description: Guía el proceso de build .msi para Windows y la promoción dev → master. Disparar cuando el usuario mencione "release", "deploy", "build de producción", "instalador", o "subirle al cliente".
---

# Deploy skill — precis (Windows MSI)

precis se distribuye como instalador `.msi` para Windows. No hay pipeline cloud — el build es local.

## Cuándo activar

- Usuario menciona: release, deploy, build de producción, instalador, "subirle al cliente".
- Antes de bumpear versión.
- Antes de cortar de `dev` a `master`.

## Flow (versión expandida del slash command `/deploy`)

1. **Verificar working tree limpio en `dev`** (`git status`).
2. **Correr la suite completa**: `pnpm lint`, `pnpm test`, `cargo test --manifest-path src-tauri/Cargo.toml`, `pnpm test:e2e`.
3. **Bumpear versión sincronizada** en los tres archivos:
   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`
4. **Commit de release**: `chore(release): vX.Y.Z`.
5. **Build**: `pnpm tauri build` → `.msi` en `src-tauri/target/release/bundle/msi/`.
6. **Smoke test** en VM/máquina sin entorno de desarrollo.
7. **PR `dev` → `master`** vía `gh pr create --base master --head dev`.
8. **Tag**: `git tag vX.Y.Z && git push origin vX.Y.Z`.
9. **Entregar** al canal acordado con el cliente.

## Gotchas conocidos

- Si las tres versiones no coinciden, Tauri falla el build con un mensaje confuso. Revisar siempre las tres.
- En Windows, la firma del `.msi` es opcional pero recomendada (`signtool`).
- Un `.msi` sin firmar dispara Windows SmartScreen — avisar al cliente de antemano.
- El binario va a `src-tauri/target/release/bundle/msi/` (no `dist/`, no `build/`).
