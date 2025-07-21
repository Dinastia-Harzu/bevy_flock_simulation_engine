# `bevy_flock_simulation_engine`

Un plugin para Bevy que consiste en un motor de físicas particular para simular boids mediante las [tres reglas básicas de Craig Reynolds](https://www.red3d.com/cwr/boids/), corrientes de viento y campos de fuerza atractivos y repulsivos.

## Cómo usar

1. Clonar repositorio
```bash
git clone https://github.com/Dinastia-Harzu/bevy_flock_simulation_engine
```
2. Crear nuevo proyecto de Rust:
```bash
cargo new <nombre del proyecto>
cd <nombre del proyecto>
```
3. Añadir los crates:
```bash
cargo add bevy
cargo add --path <ruta al repositorio clonado>
```
4. Copiar archivo de ejemplo `examples/bogstandard.rs` a `src/main.rs`:
```bash
cp <ruta al repositorio>/examples/bogstandard src/main.rs
```
5. Copiar carpeta de ejemplo de assets:
```bash
cp -r <ruta al repositorio>/assets_example/ assets/
```
6. Ejecutar con `cargo run` o `cargo run --release`

## Casos de uso

Se puede extender la funcionalidad del motor como se puede hacer con cualquier otro plugin de Bevy.
