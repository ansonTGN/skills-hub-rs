# Skills Hub Rust

<p align="center">
  <strong>Rust-native manager for Agent Skills — install once, sync across AI coding tools.</strong>
</p>

<p align="center">
  <a href="https://github.com/ansonTGN/skills-hub-rs/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/ansonTGN/skills-hub-rs/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/ansonTGN/skills-hub-rs/actions/workflows/desktop-ci.yml"><img alt="Desktop CI" src="https://github.com/ansonTGN/skills-hub-rs/actions/workflows/desktop-ci.yml/badge.svg"></a>
  <a href="https://github.com/ansonTGN/skills-hub-rs/actions/workflows/security-audit.yml"><img alt="Security audit" src="https://github.com/ansonTGN/skills-hub-rs/actions/workflows/security-audit.yml/badge.svg"></a>
  <a href="https://github.com/ansonTGN/skills-hub-rs/releases"><img alt="Release" src="https://img.shields.io/github/v/release/ansonTGN/skills-hub-rs?include_prereleases"></a>
  <a href="LICENSE"><img alt="MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

`skills-hub-rs` es un gestor independiente de **Agent Skills** escrito en Rust. Mantiene una biblioteca central de Skills y la sincroniza con múltiples agentes y herramientas de programación.

La misma lógica de negocio alimenta dos superficies:

- **CLI nativa en Rust** para automatización y scripting.
- **Desktop Tauri 2** para gestión visual de Skills, targets, tags, búsqueda, scan y papelera.

> **Estado actual:** `v0.2.0-alpha.2` es una **pre-release** pública para Linux x86_64 y Windows x86_64. Incluye CLI, aplicación Desktop, instaladores, checksums SHA-256, SBOM CycloneDX 1.5 y attestations verificables de GitHub Actions.

> El proyecto está inspirado en el modelo funcional de Skills Hub, pero es una implementación Rust independiente; no es un fork línea por línea.

## Modelo operativo

```text
Instalar una vez
      │
      ▼
~/.skillshub-rs
      │
      ├── Codex
      ├── Claude Code
      ├── Cursor
      ├── Gemini CLI
      ├── GitHub Copilot
      ├── Kimi Code CLI
      └── ... 47 adaptadores integrados
```

La biblioteca central conserva el estado gestionado; cada target recibe el Skill mediante `symlink`, `junction` o `copy` según plataforma, herramienta y política elegida.

## Funcionalidades

| Área | Capacidades |
| --- | --- |
| Biblioteca | Hub central persistente en `~/.skillshub-rs` y SQLite embebido |
| Instalación | Directorios locales, repositorios Git, branch/tag y subdirectorios |
| Descubrimiento | Búsqueda recursiva de `SKILL.md` y `scan` de herramientas instaladas |
| Integraciones | 47 adaptadores integrados + herramientas personalizadas |
| Sincronización | Scope global/proyecto; `auto`, `symlink`, `junction`, `copy` |
| Ciclo de vida | Enable/disable, update, unsync y eliminación segura |
| Organización | Tags y búsqueda en `skills.sh` |
| Recuperación | Papelera con restore y purga; retención lógica de 30 días |
| Desktop | Dashboard, Skills, Tools, instalación, búsqueda, scan y papelera |
| Supply chain | RustSec, SHA-256, CycloneDX 1.5 y artifact attestations |

## Descarga

Release actual:

https://github.com/ansonTGN/skills-hub-rs/releases/tag/v0.2.0-alpha.2

### Linux — AppImage

```bash
VERSION=v0.2.0-alpha.2

gh release download "$VERSION"   --repo ansonTGN/skills-hub-rs   --pattern 'skills-hub-rs-desktop-linux-x86_64.AppImage'   --pattern 'skills-hub-rs-desktop-linux-x86_64.AppImage.sha256'

sha256sum -c skills-hub-rs-desktop-linux-x86_64.AppImage.sha256
chmod +x skills-hub-rs-desktop-linux-x86_64.AppImage
./skills-hub-rs-desktop-linux-x86_64.AppImage
```

### Debian/Ubuntu — `.deb`

```bash
VERSION=v0.2.0-alpha.2

gh release download "$VERSION"   --repo ansonTGN/skills-hub-rs   --pattern 'skills-hub-rs-desktop-linux-x86_64.deb'   --pattern 'skills-hub-rs-desktop-linux-x86_64.deb.sha256'

sha256sum -c skills-hub-rs-desktop-linux-x86_64.deb.sha256
sudo apt install ./skills-hub-rs-desktop-linux-x86_64.deb
```

### Linux — CLI

```bash
VERSION=v0.2.0-alpha.2
mkdir -p skills-hub-rs-cli
cd skills-hub-rs-cli

gh release download "$VERSION"   --repo ansonTGN/skills-hub-rs   --pattern 'skills-hub-rs-linux-x86_64.tar.gz'   --pattern 'skills-hub-rs-linux-x86_64.tar.gz.sha256'

sha256sum -c skills-hub-rs-linux-x86_64.tar.gz.sha256
tar -xzf skills-hub-rs-linux-x86_64.tar.gz
./skills-hub-rs --help
```

### Windows

- `skills-hub-rs-windows-x86_64.zip` — CLI portable.
- `skills-hub-rs-desktop-windows-x86_64-setup.exe` — instalador NSIS.
- `skills-hub-rs-desktop-windows-x86_64.msi` — instalador MSI.

Cada artefacto incluye su `.sha256`.

## Primeros pasos con la CLI

Para probar sin tocar la configuración real:

```bash
export TEST_HUB=/tmp/skills-hub-rs-test
rm -rf "$TEST_HUB"

skills-hub-rs --home "$TEST_HUB" init
skills-hub-rs --home "$TEST_HUB" tools
skills-hub-rs --home "$TEST_HUB" add-local examples/demo-skill --tag prueba
skills-hub-rs --home "$TEST_HUB" list
skills-hub-rs --home "$TEST_HUB" show demo-skill
```

Ubicación real por defecto:

```text
~/.skillshub-rs
```

### Instalar y sincronizar un Skill local

```bash
skills-hub-rs add-local /ruta/al/skill   --tag rust   --tool codex
```

### Scope de proyecto

```bash
skills-hub-rs sync demo-skill   --tool codex   --scope project   --project /ruta/al/proyecto
```

### Claude Code y Cursor

```bash
skills-hub-rs sync demo-skill --tool claude_code
skills-hub-rs sync demo-skill --tool cursor
```

Cursor fuerza `copy`. Para forzar copia en otra herramienta:

```bash
skills-hub-rs sync demo-skill   --tool codex   --mode copy   --overwrite
```

### Instalar desde Git

```bash
skills-hub-rs add-git   https://github.com/usuario/repositorio.git   --tool codex   --tag github
```

Con branch/tag:

```bash
skills-hub-rs add-git   https://github.com/usuario/repositorio.git   --ref v1.0.0
```

Con subdirectorio:

```bash
skills-hub-rs add-git   https://github.com/usuario/repositorio.git   --subdir skills
```

### Actualizar

```bash
skills-hub-rs update demo-skill
skills-hub-rs update
```

### Scan

```bash
skills-hub-rs scan
skills-hub-rs scan --tool codex
```

### Enable / disable

```bash
skills-hub-rs disable demo-skill
skills-hub-rs enable demo-skill
```

### Papelera

```bash
skills-hub-rs remove demo-skill
skills-hub-rs recycle list
skills-hub-rs recycle restore <ID>
skills-hub-rs recycle purge
skills-hub-rs recycle purge --all
skills-hub-rs remove demo-skill --permanent
```

### Herramientas personalizadas

```bash
skills-hub-rs custom-tool add mi_agente   --label "Mi agente"   --global-dir '~/.mi-agente/skills'   --project-dir '.mi-agente/skills'   --mode auto

skills-hub-rs sync demo-skill --tool mi_agente
```

### Búsqueda en skills.sh

```bash
skills-hub-rs search rust
skills-hub-rs search security --limit 10 --json
```

## Desktop Tauri

La GUI delega en el mismo motor Rust que la CLI:

```text
Desktop WebView
      │
      │ Tauri IPC
      ▼
skills-hub-rs-desktop
      │
      │ path dependency
      ▼
skills-hub-rs core
  ├─ manager
  ├─ SQLite
  ├─ filesystem
  ├─ Git
  ├─ tools
  └─ skills.sh
```

La interfaz actual incluye dashboard, gestión de Skills, Tools, instalación local/Git, sync/unsync, enable/disable, update, búsqueda online, scan, custom tools y papelera.

Consulta [`docs/V0.2.0_TAURI.md`](docs/V0.2.0_TAURI.md) para detalles técnicos.

## Adaptadores integrados

El core incluye **47 adaptadores**. La fuente de verdad está en [`src/tools.rs`](src/tools.rs):

`Cursor`, `Claude Code`, `Codex`, `DeepSeek Harness`, `OpenCode`, `Antigravity`, `Amp`, `Kimi Code CLI`, `Augment`, `OpenClaw`, `Copaw`, `Cline`, `CodeBuddy`, `CodeWhale`, `WorkBuddy`, `Command Code`, `Continue`, `Crush`, `Junie`, `iFlow CLI`, `Kiro CLI`, `Kode`, `MCPJam`, `Mistral Vibe`, `Mux`, `OpenClaude IDE`, `OpenHands`, `Pi`, `Qoder`, `QoderWork`, `Qwen Code`, `Trae`, `Trae CN`, `Zencoder`, `Neovate`, `Pochi`, `AdaL`, `Kilo Code`, `Roo Code`, `Goose`, `Gemini CLI`, `GitHub Copilot`, `Clawdbot`, `Droid`, `Windsurf`, `MoltBot`, `Hermes Agent`.

## Comandos principales

```text
init
list
show
files
tools
scan
add-local
add-git
sync
unsync
disable
enable
update
remove
recycle
search
tag
custom-tool
```

Ayuda:

```bash
skills-hub-rs --help
skills-hub-rs <comando> --help
```

## Seguridad del filesystem

- evita sincronizaciones con origen/destino solapados;
- no sobrescribe targets salvo `--overwrite`;
- conserva targets físicos compartidos mientras exista otra relación propietaria;
- elimina symlinks/junctions como enlaces, sin recorrer deliberadamente su destino;
- nunca borra el origen local original al eliminar un Skill gestionado;
- las actualizaciones locales preparan el contenido antes del reemplazo;
- Cursor usa `copy` por diseño.

## Seguridad del Desktop

`v0.2.0-alpha.2` endurece la WebView Tauri mediante:

- CSP restrictiva con `default-src 'self'`;
- scripts/estilos limitados al bundle;
- `object-src 'none'`;
- `base-uri 'none'`;
- `form-action 'none'`;
- `frame-ancestors 'none'`;
- `X-Content-Type-Options: nosniff`;
- `Permissions-Policy` con cámara, micrófono y geolocalización deshabilitados.

## Supply chain

La pipeline incorpora:

1. `cargo fmt`, `cargo check`, Clippy con `-D warnings`, tests y smoke tests.
2. Auditoría RustSec para core y Desktop.
3. Builds independientes Linux/Windows.
4. SHA-256 para entregables.
5. SBOM **CycloneDX 1.5** para core y Desktop.
6. GitHub artifact attestations para provenance y asociación de SBOM.

SBOM publicados:

```text
skills-hub-rs-sbom.cdx.json
skills-hub-rs-desktop-sbom.cdx.json
```

> El SBOM Desktop representa el grafo Rust/Cargo. No pretende todavía inventariar exhaustivamente todas las bibliotecas nativas de GTK/WebKit.

### Verificar checksum

```bash
sha256sum -c skills-hub-rs-linux-x86_64.tar.gz.sha256
```

### Verificar attestation

```bash
gh attestation verify   skills-hub-rs-linux-x86_64.tar.gz   --repo ansonTGN/skills-hub-rs
```

Desktop:

```bash
gh attestation verify   skills-hub-rs-desktop-linux-x86_64.AppImage   --repo ansonTGN/skills-hub-rs
```

SBOM:

```bash
gh attestation verify   skills-hub-rs-sbom.cdx.json   --repo ansonTGN/skills-hub-rs
```

## Artefactos de `v0.2.0-alpha.2`

La release publica **20 assets** contando entregables y checksums:

| Plataforma | Entregables principales |
| --- | --- |
| Linux CLI | binario raw + `.tar.gz` |
| Windows CLI | `.exe` raw + `.zip` |
| Linux Desktop | `.AppImage` + `.deb` |
| Windows Desktop | NSIS `.exe` + `.msi` |
| Supply chain | SBOM core + SBOM Desktop |

Cada uno de esos diez artefactos principales dispone de su `.sha256`.

## Compilar desde código fuente

Requisitos principales:

- Rust `1.96.0`;
- Git en `PATH`;
- toolchain nativo de la plataforma.

Ubuntu:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup toolchain install 1.96.0 --profile minimal --component rustfmt,clippy
rustup default 1.96.0
```

Core:

```bash
git clone https://github.com/ansonTGN/skills-hub-rs.git
cd skills-hub-rs
./scripts/validate-local-linux.sh
```

Desktop Ubuntu 24.04:

```bash
sudo apt install -y   build-essential   curl   wget   file   pkg-config   libssl-dev   libgtk-3-dev   libwebkit2gtk-4.1-dev   librsvg2-dev   libayatana-appindicator3-dev

cargo install tauri-cli --version 2.11.4 --locked
./scripts/validate-desktop-linux.sh
```

Ejecutar la GUI contra un hub aislado:

```bash
export SKILLS_HUB_HOME=/tmp/skills-hub-rs-gui-test
cargo run --manifest-path desktop/src-tauri/Cargo.toml
```

## CI/CD

El repositorio mantiene:

- [`ci.yml`](.github/workflows/ci.yml) — core Linux/Windows.
- [`desktop-ci.yml`](.github/workflows/desktop-ci.yml) — Desktop Linux/Windows.
- [`security-audit.yml`](.github/workflows/security-audit.yml) — RustSec en PR, `main`, manual y semanal.
- [`release.yml`](.github/workflows/release.yml) — builds, SBOM, attestations y GitHub Release.

## Roadmap hacia `v0.2.0`

Tras `alpha.2`, los siguientes bloques naturales son:

- selectores nativos de archivos/directorios en la GUI;
- mover operaciones Git/filesystem potencialmente lentas fuera del hilo UI;
- normalizar nombres y metadatos de packaging;
- ampliar smoke tests de instaladores;
- evaluar firma de código y mecanismo de actualización.

## Licencia

MIT. Consulta [`LICENSE`](LICENSE).
