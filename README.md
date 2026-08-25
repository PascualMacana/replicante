# replicante

Un programa en Rust que **se copia a sí mismo**: el binario lleva embebido su genoma (las fuentes) y puede escribir un proyecto Cargo hijo que, al compilarse, puede hacer lo mismo.

No es un LLM. No se reentrena. No se propaga por la red. Es un constructor a lo von Neumann: la máquina lleva adentro la descripción de cómo reconstruirse.

```
generación 0  ──spawn──►  generación 1  ──spawn──►  generación 2
   linaje 0                  linaje 0.1               linaje 0.1.2
```

## Qué es (y qué no)

| Concepto | Esto |
|---|---|
| **Quine** | Un programa que *imprime* su fuente. `replicante genome` se acerca. |
| **Constructor auto-reproductor** | Un programa que *escribe* una copia funcional de sí. Esto. |
| **Auto-mejora (máquina de Gödel)** | Produce una versión *mejor*, no solo una copia. Esto no mejora: solo hereda y cuenta generaciones. |
| **Gusano / malware** | Se copia a otros discos, procesos o redes sin que se lo pidan. Esto **no**. |

El hijo nace solo donde vos indicás, un directorio por invocación. Hace falta `rustc`/`cargo` para que el hijo "viva" (el genoma necesita un compilador, como el ADN una célula).

## Uso

```bash
cargo build --release
./target/release/replicante identity
./target/release/replicante spawn ./hijo --build
./hijo/target/debug/replicante identity
./hijo/target/debug/replicante spawn ./nieto --build
./nieto/target/debug/replicante identity
```

Comandos:

```
replicante              ayuda
replicante identity     generación, linaje, archivos del genoma
replicante genome       imprime las fuentes embebidas
replicante spawn <dir>  escribe el proyecto hijo
                 --build   compila al hijo con cargo
                 --force   pisa un hijo anterior
```

## Cómo funciona

En compile-time, `include_str!` mete cada archivo del repo adentro del binario:

```rust
const GENOME: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../Cargo.toml")),
    ("src/main.rs", include_str!("main.rs")),
    // ...
];
```

`spawn` los escribe a disco y parchea dos constantes en `src/main.rs`:

```rust
const GENERATION: u32 = 0;
const LINEAGE: &str = "0";
```

El hijo queda con `GENERATION = 1` y `LINEAGE = "0.1"`. Cuando *ese* hijo se compila, su `include_str!` ya captura la fuente nueva. La herencia es real: no es copiar el ejecutable (el fenotipo), es copiar y mutar el genoma.

## Límites a propósito

- Un solo hijo por corrida. No hay loop, cron, ni fork bomb.
- No toca `$HOME`, `/`, `/usr`, `/etc`, ni el directorio desde el que corrés.
- `--force` solo borra un destino que ya parece un `replicante`.
- Cero red: no crea repos, no pega a APIs, no se envía por mail.

## Por qué existe

Quise tener en las manos la diferencia entre **auto-replicarse** y **auto-mejorarse**. Esto es lo primero. Lo segundo es [mejorante](https://github.com/PascualMacana/mejorante): el hijo no solo nace, nace *mejor*.
