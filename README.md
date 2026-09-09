# Skills Hub Rust

Gestor nativo en Rust de **Agent Skills** inspirado en el modelo funcional de
Skills Hub: instala cada Skill una sola vez en una biblioteca central y lo
sincroniza con distintos agentes/herramientas de programación.

> Estado: `v0.1.0` CLI. El objetivo de esta primera versión es validar el núcleo
> local antes de añadir una interfaz Tauri. No es un fork línea por línea del
> proyecto original.

## Funciones incluidas

- biblioteca central persistente, por defecto `~/.skillshub-rs`;
- SQLite embebido para Skills, tags, targets, herramientas personalizadas y papelera;
- instalación desde carpetas locales;
- instalación desde repositorios Git;
- descubrimiento recursivo de directorios que contienen `SKILL.md`;
- **47 adaptadores** integrados para herramientas de IA;
- sincronización global o por proyecto;
- `symlink`/`junction`/`copy`, con fallback automático a copia;
- Cursor fuerza `copy`;
- soporte de `KIMI_CODE_HOME` para Kimi Code CLI;
- tags;
- habilitar/deshabilitar Skills sin borrar la copia central;
- actualización desde origen local o Git;
- papelera recuperable durante 30 días;
- búsqueda en `skills.sh`;
- herramientas personalizadas;
- pruebas de integración y smoke tests Linux/Windows;
- GitHub Actions para CI y Releases binarias.

## Requisitos para compilar

- Rust stable (`rustup`, `cargo`, `rustc`);
- Git disponible en `PATH` para `add-git` y actualizaciones Git.

En Ubuntu:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup toolchain install 1.96.0 --profile minimal --component rustfmt,clippy
rustup default 1.96.0
```

## Primera validación local

Si estás usando el ZIP que acompaña a esta entrega:

```bash
unzip skills-hub-rs-v0.1.0-source.zip
cd skills-hub-rs
./scripts/validate-local-linux.sh
```

El script ejecuta todo el pipeline local: `rustfmt`, `Cargo.lock`, `check`,
`clippy`, tests, build release, smoke test y empaquetado Linux.

Equivalente manual:

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo generate-lockfile
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
./scripts/smoke-test.sh ./target/release/skills-hub-rs
```

El último comando debe terminar con:

```text
SMOKE TEST OK
```

El ejecutable estará en:

```text
target/release/skills-hub-rs
```

## Probar sin tocar tu configuración real

Toda la CLI acepta `--home`. Así puedes usar un sandbox:

```bash
export TEST_HUB=/tmp/skills-hub-rs-test
rm -rf "$TEST_HUB"

./target/release/skills-hub-rs --home "$TEST_HUB" init
./target/release/skills-hub-rs --home "$TEST_HUB" tools
./target/release/skills-hub-rs --home "$TEST_HUB" add-local examples/demo-skill --tag prueba
./target/release/skills-hub-rs --home "$TEST_HUB" list
./target/release/skills-hub-rs --home "$TEST_HUB" show demo-skill
```

## Sincronizar con Codex

Después de validar el sandbox puedes probar con tu configuración real:

```bash
./target/release/skills-hub-rs add-local /ruta/al/skill \
  --tag rust \
  --tool codex
```

La copia central queda en `~/.skillshub-rs/skills/<skill>` y Codex recibe el
Skill en su directorio global configurado.

Para scope de proyecto:

```bash
./target/release/skills-hub-rs sync demo-skill \
  --tool codex \
  --scope project \
  --project /ruta/al/proyecto
```

## Claude Code y Cursor

```bash
./target/release/skills-hub-rs sync demo-skill --tool claude_code
./target/release/skills-hub-rs sync demo-skill --tool cursor
```

Cursor usa `copy` deliberadamente. Para forzar copia con cualquier otra herramienta:

```bash
./target/release/skills-hub-rs sync demo-skill --tool codex --mode copy --overwrite
```

## Instalar desde Git

```bash
./target/release/skills-hub-rs add-git \
  https://github.com/usuario/repositorio.git \
  --tool codex \
  --tag github
```

Con rama/tag:

```bash
./target/release/skills-hub-rs add-git \
  https://github.com/usuario/repositorio.git \
  --ref v1.0.0
```

Con subdirectorio:

```bash
./target/release/skills-hub-rs add-git \
  https://github.com/usuario/repositorio.git \
  --subdir skills
```

## Actualizar

```bash
./target/release/skills-hub-rs update demo-skill
./target/release/skills-hub-rs update
```

Los Skills locales se comparan contra su carpeta original. Los Git se vuelven a
obtener del repositorio y se resincronizan cuando cambia el contenido.

## Scan

```bash
./target/release/skills-hub-rs scan
./target/release/skills-hub-rs scan --tool codex
```

`scan` descubre Skills presentes en directorios de herramientas. Para gestionar
uno de ellos, impórtalo con `add-local <ruta-detectada>`.

## Habilitar y deshabilitar

```bash
./target/release/skills-hub-rs disable demo-skill
./target/release/skills-hub-rs enable demo-skill
```

Deshabilitar elimina los targets activos, pero conserva la copia gestionada y su
configuración en SQLite.

## Papelera

El borrado normal es recuperable:

```bash
./target/release/skills-hub-rs remove demo-skill
./target/release/skills-hub-rs recycle list
./target/release/skills-hub-rs recycle restore <ID>
```

Purgar elementos con más de 30 días:

```bash
./target/release/skills-hub-rs recycle purge
```

Eliminar todo lo contenido en la papelera:

```bash
./target/release/skills-hub-rs recycle purge --all
```

Borrado inmediato y permanente:

```bash
./target/release/skills-hub-rs remove demo-skill --permanent
```

## Herramienta personalizada

```bash
./target/release/skills-hub-rs custom-tool add mi_agente \
  --label "Mi agente" \
  --global-dir '~/.mi-agente/skills' \
  --project-dir '.mi-agente/skills' \
  --mode auto

./target/release/skills-hub-rs sync demo-skill --tool mi_agente
```

## Búsqueda online

```bash
./target/release/skills-hub-rs search rust
./target/release/skills-hub-rs search security --limit 10 --json
```

## Publicación en GitHub

No es necesario versionar ejecutables dentro del historial Git. Este repositorio
incluye `.github/workflows/release.yml` para publicar artefactos en **GitHub Releases**.

Primero sube y valida `main`:

```bash
git init
git add .
git commit -m "Initial Rust implementation"
git branch -M main
git remote add origin https://github.com/ansonTGN/skills-hub-rs.git
git push -u origin main
```

Tras la compilación local, comprueba que `Cargo.lock` existe y añádelo al commit.
No publiques una release sin ese lockfile.

Cuando CI esté verde:

```bash
git tag -a v0.1.0 -m "skills-hub-rs v0.1.0"
git push origin v0.1.0
```

El workflow crea automáticamente:

- `skills-hub-rs-linux-x86_64.tar.gz`
- `skills-hub-rs-linux-x86_64.tar.gz.sha256`
- `skills-hub-rs-windows-x86_64.zip`
- `skills-hub-rs-windows-x86_64.zip.sha256`

Los binarios se publican en la página **Releases** del repositorio y se pueden
descargar sin instalar Rust.

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

Usa:

```bash
skills-hub-rs --help
skills-hub-rs <comando> --help
```

## Diferencias respecto al Skills Hub desktop actual

Esta versión reproduce el núcleo local de gestión y distribución de Skills, pero
**v0.1.0 no pretende todavía paridad total con la aplicación desktop upstream**.
Quedan fuera de esta fase:

- UI React/Tauri;
- actualizador de la propia aplicación;
- scheduler del sistema para actualizaciones automáticas;
- sincronización multi-dispositivo mediante repositorio Git;
- OAuth/token vault para dicha sincronización;
- merge de tres vías y resolución interactiva de conflictos entre dispositivos.

Es preferible estabilizar primero el motor Rust local y el formato de datos antes
de añadir esas capas.

## Seguridad de filesystem

- no se sincroniza si origen y destino se solapan;
- un target existente no se sobrescribe salvo operación explícita con `--overwrite`;
- al borrar un target compartido por varias relaciones se conserva mientras exista otro propietario;
- enlaces simbólicos y junctions se eliminan como enlaces, sin recorrer deliberadamente su destino;
- el origen local original nunca se elimina al quitar un Skill gestionado.

## Licencia

MIT.
