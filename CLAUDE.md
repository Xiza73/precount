# precis — Generador de XLS para SISCONT

> Aplicación desktop (Tauri + Rust + React) que transforma archivos Excel/CSV de entrada y data manual en archivos `.xls` listos para importar en **SISCONT** (software contable peruano).

---

## 1. Contexto del proyecto

**precis** es una herramienta para un cliente real que necesita generar planillas Excel con el formato exacto que espera SISCONT (módulos de Compras, Ventas, Planillas, etc.) a partir de archivos de origen heterogéneos y/o data manual.

El equipo NO es experto en contabilidad — los formatos y reglas de validación se van descubriendo y validando con el cliente durante el desarrollo. Asumir que cualquier formato SISCONT que no esté documentado en el repo todavía está por confirmar.

**Restricciones clave (no negociables):**

- Los archivos de **entrada** (xls, csv) **NO se guardan** en disco — se procesan en memoria y se descartan.
- El archivo de **salida** **TAMPOCO se persiste** — se devuelve al frontend como `base64` para que el usuario lo descargue.
- La aplicación es **100% local**: sin red, sin servidor HTTP, sin base de datos, sin auth.
- Plataforma objetivo: **Windows** (otras plataformas no descartadas, pero no priorizadas).

## 2. Usuarios y alcance (MVP)

**Usuario final:** asistente contable del cliente. Carga archivos Excel del ERP/punto-de-venta, completa datos manuales si faltan, descarga el XLS listo para SISCONT.

**MVP — Módulo 1: Compras (Registro de Compras SUNAT)**

- Carga de archivo de comprobantes de compra (xls/csv) en memoria.
- Formulario manual para correcciones o entradas que no vinieron en el archivo.
- Validaciones por columna (RUC, tipo de comprobante, fechas, montos).
- Generación del XLS de salida con el layout exacto que importa SISCONT.
- Descarga en el frontend (base64 → `Blob` → `a[download]`).

**Diferido (post-MVP):** Ventas, Planillas, otros módulos SISCONT. La arquitectura debe ser **modular desde el día uno** para que sumar módulos sea trivial.

## 3. Stack y herramientas

| Capa             | Tecnología                                     |
| ---------------- | ---------------------------------------------- |
| Shell desktop    | Tauri 2.x                                      |
| Backend nativo   | Rust                                           |
| Frontend         | React 19 + TypeScript (strict)                 |
| Build frontend   | Vite                                           |
| Package manager  | pnpm                                           |
| Linter/Formatter | Biome                                          |
| Tests frontend   | Vitest                                         |
| Tests Rust       | `cargo test`                                   |
| Tests E2E        | Playwright                                     |
| Parsing xls      | `calamine` (read) + `rust_xlsxwriter` (write)  |

## 4. Comandos clave

```bash
# Instalar dependencias
pnpm install

# Dev (Vite + Tauri en watch)
pnpm tauri dev

# Build release Windows (.msi)
pnpm tauri build

# Lint + format (Biome)
pnpm lint        # biome check
pnpm format      # biome format --write

# Tests
pnpm test                                               # Vitest (frontend)
cargo test --manifest-path src-tauri/Cargo.toml         # tests Rust
pnpm test:e2e                                           # Playwright
```

> Si los scripts todavía no existen en `package.json`, configurarlos al hacer `pnpm create tauri-app`.

## 5. Convenciones de código

**Commits:** [Conventional Commits](https://www.conventionalcommits.org/) — `feat:`, `fix:`, `refactor:`, `chore:`, `docs:`, `test:`, `build:`. Sin scope obligatorio, pero recomendado cuando aporta (`feat(compras):`).

**Branching:**

- `dev` — rama principal de desarrollo. Todo PR mergea aquí.
- `master` — rama de releases (deploys). Solo recibe merges desde `dev` cuando se corta una versión.
- Feature branches: `feat/<scope>`, `fix/<scope>` desde `dev`.

**TypeScript:** `strict: true`. Sin `any` salvo casos justificados con comentario.

**React 19:** sin `useMemo`/`useCallback` (lo hace React Compiler). Componentes funcionales. No hay Next, no hay Server Components.

**Rust:** `clippy` clean obligatorio. `cargo fmt` antes de commit. Sin `unwrap()`/`expect()` en código de producción — todo va con `Result<T, AppError>`.

**Naming:**

- Archivos: `kebab-case.ts`, `kebab-case.tsx`, `snake_case.rs`.
- Componentes y tipos TS: `PascalCase`.
- Funciones y variables: `camelCase` (TS), `snake_case` (Rust).
- Constantes: `SCREAMING_SNAKE_CASE`.

## 6. Estructura del repositorio

```
precis/
├── src/                       # Frontend React + TS
│   ├── modules/
│   │   └── compras/           # Módulo Compras (MVP)
│   ├── components/            # Componentes compartidos
│   ├── lib/                   # Helpers TS (validaciones, fechas, base64...)
│   └── main.tsx
├── src-tauri/                 # Backend Rust + Tauri config
│   ├── src/
│   │   ├── commands/          # Tauri commands invocables desde JS
│   │   │   └── compras.rs
│   │   ├── xls/               # Parsing + generación xls
│   │   ├── domain/            # Tipos y reglas de negocio (RUC, comprobantes...)
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/                     # Tests Playwright E2E
├── biome.json
├── package.json
└── tsconfig.json
```

## 7. Reglas de trabajo con Claude

### Qué SÍ hacer

- Antes de implementar un módulo SISCONT nuevo, **validar el formato del XLS de salida con el cliente** (columnas, tipos, codificaciones). No asumas.
- Input y output xls van en memoria (`Vec<u8>` en Rust, `Uint8Array`/`base64` en TS). Nunca tocar `fs::write`/`fs::read` con datos del usuario.
- Estructura modular: cada módulo SISCONT (compras, ventas, planillas...) es independiente y reutiliza primitivas compartidas (validación de RUC, parseo de fechas, helpers de moneda).
- Cada nuevo Tauri command lleva su `cargo test`. Cada componente con lógica lleva su `.test.tsx` en Vitest.
- Commits siguen [Conventional Commits](#convenciones-de-código). PRs apuntan siempre a `dev`.

### Qué NO hacer

- **NO persistir archivos** del usuario en disco — ni input ni output. Si hace falta intermedio, `tempfile` + cleanup garantizado en `Drop`.
- **NO agregar Nest**, ni un backend HTTP, ni base de datos sin discutirlo primero. La app es local por diseño.
- NO usar `useMemo`/`useCallback` (React 19 + Compiler los hace por vos).
- NO inventar formatos SISCONT. Si no sabés cómo es una columna, marcala como `TODO` con un test que falle (`#[test] #[ignore]`).
- NO añadir attribution AI en commits (`Co-Authored-By: Claude` está prohibido).
- NO hacer `git push --force`, `git reset --hard`, ni mergear a `master` sin pasar por PR desde `dev`.

---

> Reglas personales del dev → ver `CLAUDE.local.md` (no commiteado).
