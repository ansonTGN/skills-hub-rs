# Checklist de validación local antes de GitHub

No subas `v0.1.0` hasta completar esta lista en el host Ubuntu.

## 1. Toolchain

```bash
cd skills-hub-rs
rustc --version
cargo --version
rustup show active-toolchain
```

El proyecto fija Rust `1.96.0` mediante `rust-toolchain.toml`.

## 2. Formato y lockfile

```bash
cargo fmt --all
cargo generate-lockfile

test -f Cargo.lock
cargo fmt --all -- --check
```

`Cargo.lock` debe incluirse en Git para que CI y Releases resuelvan exactamente
las mismas dependencias validadas localmente.

## 3. Compilación estática

```bash
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
```

No continúes si Clippy devuelve warnings o errores.

## 4. Tests Rust

```bash
cargo test --all-targets --locked
```

Todos deben finalizar `ok`.

## 5. Binario release

```bash
cargo build --release --locked
file target/release/skills-hub-rs
./target/release/skills-hub-rs --version
./target/release/skills-hub-rs --help
```

## 6. Smoke test aislado

```bash
./scripts/smoke-test.sh ./target/release/skills-hub-rs
```

Resultado obligatorio:

```text
SMOKE TEST OK
```

Este test usa un directorio temporal; no necesita escribir en `~/.codex`,
`~/.claude` ni otras rutas reales.

## 7. Prueba controlada con Codex

Crea un home de Skills Hub separado:

```bash
export SKILLS_HUB_HOME="$HOME/.skillshub-rs-test"
rm -rf "$SKILLS_HUB_HOME"

./target/release/skills-hub-rs init
./target/release/skills-hub-rs tools --detected
./target/release/skills-hub-rs add-local examples/demo-skill --tag local-test
./target/release/skills-hub-rs show demo-skill
```

No sincronices todavía con una herramienta real si deseas mantener la prueba
completamente aislada.

## 8. Prueba real opcional con Codex

Antes de ejecutar, revisa que no exista ya un Skill con el mismo nombre en el
target. La sincronización es no destructiva salvo `--overwrite`.

```bash
unset SKILLS_HUB_HOME
./target/release/skills-hub-rs add-local examples/demo-skill --tag local-test
./target/release/skills-hub-rs sync demo-skill --tool codex
./target/release/skills-hub-rs show demo-skill
```

Para deshacer:

```bash
./target/release/skills-hub-rs unsync demo-skill --tool codex
./target/release/skills-hub-rs remove demo-skill --permanent
```

## 9. Estado Git previo al push

```bash
git status --short
git diff --check
```

Asegúrate de que `Cargo.lock` está incluido:

```bash
git ls-files --error-unmatch Cargo.lock
```

## 10. Subida a GitHub

Solo después de 1–9:

```bash
git add .
git commit -m "Initial Rust implementation"
git branch -M main
git remote add origin https://github.com/ansonTGN/skills-hub-rs.git
git push -u origin main
```

Espera a que `CI` sea verde en Ubuntu y Windows.

## 11. Primera release

```bash
git tag -a v0.1.0 -m "skills-hub-rs v0.1.0"
git push origin v0.1.0
```

La release solo se publica después de compilar y ejecutar el smoke test en
Linux y Windows. Debe contener:

```text
skills-hub-rs-linux-x86_64
skills-hub-rs-linux-x86_64.sha256
skills-hub-rs-linux-x86_64.tar.gz
skills-hub-rs-linux-x86_64.tar.gz.sha256
skills-hub-rs-windows-x86_64.exe
skills-hub-rs-windows-x86_64.exe.sha256
skills-hub-rs-windows-x86_64.zip
skills-hub-rs-windows-x86_64.zip.sha256
```
