//! Constructor auto-reproductor.
//!
//! El binario lleva su genoma (fuentes) embebido en compile-time.
//! `spawn <dir>` escribe un proyecto Cargo hijo que, al compilares, puede hacer lo mismo.

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

const GENERATION: u32 = 0;
const LINEAGE: &str = "0";

const GENOME: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../Cargo.toml")),
    ("src/main.rs", include_str!("main.rs")),
    ("README.md", include_str!("../README.md")),
    (".gitignore", include_str!("../.gitignore")),
    ("LICENSE", include_str!("../LICENSE")),
];

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None | Some("help") | Some("-h") | Some("--help") => help(),
        Some("identity") | Some("id") => identity(),
        Some("genome") => print_genome(),
        Some("spawn") => {
            let dest = args.next().unwrap_or_else(|| {
                eprintln!("uso: replicante spawn <directorio> [--build] [--force]");
                process::exit(2);
            });
            let mut force = false;
            let mut build = false;
            for flag in args {
                match flag.as_str() {
                    "--force" => force = true,
                    "--build" => build = true,
                    other => {
                        eprintln!("flag desconocida: {other}");
                        process::exit(2);
                    }
                }
            }
            if let Err(e) = spawn(Path::new(&dest), force, build) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some(other) => {
            eprintln!("comando desconocido: {other}\n");
            help();
            process::exit(2);
        }
    }
}

fn help() {
    println!(
        "\
replicante — constructor auto-reproductor (generación {GENERATION}, linaje {LINEAGE})

  replicante identity          generación, linaje, archivos del genoma
  replicante genome            imprime las fuentes embebidas
  replicante spawn <dir>       escribe un proyecto Cargo hijo
                   --build     compila al hijo con cargo
                   --force     pisa un hijo anterior

El hijo hereda el genoma y suma una generación. No se copia por la red,
no pisa el directorio actual, un solo hijo por corrida."
    );
}

fn identity() {
    println!("replicante");
    println!("generación  {GENERATION}");
    println!("linaje      {LINEAGE}");
    println!("archivos    {}", GENOME.len());
    for (name, body) in GENOME {
        println!("  {name:<16}  {} bytes", body.len());
    }
}

fn print_genome() {
    for (i, (name, body)) in GENOME.iter().enumerate() {
        if i > 0 {
            println!();
        }
        println!("===== {name} =====");
        print!("{body}");
        if !body.ends_with('\n') {
            println!();
        }
    }
}

fn spawn(dest: &Path, force: bool, build: bool) -> Result<(), Box<dyn Error>> {
    let dest = normalize_dest(dest)?;
    assert_safe_dest(&dest)?;
    prepare_dest(&dest, force)?;

    let child_main = rewrite_main(include_str!("main.rs"))?;
    let next_gen = GENERATION + 1;
    let next_lineage = format!("{LINEAGE}.{next_gen}");

    for (rel, contents) in GENOME {
        let body = if *rel == "src/main.rs" {
            child_main.clone()
        } else {
            (*contents).to_string()
        };
        let path = dest.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, body)?;
        println!("escribió {}", path.display());
    }

    println!(
        "hijo generación {next_gen} linaje {next_lineage} → {}",
        dest.display()
    );

    if build {
        let status = Command::new("cargo")
            .arg("build")
            .current_dir(&dest)
            .status()?;
        if !status.success() {
            return Err("cargo build del hijo falló".into());
        }
        println!("hijo compilado: {}/target/debug/replicante", dest.display());
    }

    Ok(())
}

fn rewrite_main(src: &str) -> Result<String, Box<dyn Error>> {
    let next_gen = GENERATION + 1;
    let next_lineage = format!("{LINEAGE}.{next_gen}");

    let from_g = format!("const GENERATION: u32 = {GENERATION};");
    let to_g = format!("const GENERATION: u32 = {next_gen};");
    if !src.contains(&from_g) {
        return Err("no encuentro GENERATION en el genoma".into());
    }

    let from_l = format!("const LINEAGE: &str = \"{LINEAGE}\";");
    let to_l = format!("const LINEAGE: &str = \"{next_lineage}\";");
    if !src.contains(&from_l) {
        return Err("no encuentro LINEAGE en el genoma".into());
    }

    Ok(src.replacen(&from_g, &to_g, 1).replacen(&from_l, &to_l, 1))
}

fn normalize_dest(dest: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if dest.as_os_str().is_empty() {
        return Err("directorio vacío".into());
    }
    if dest.is_absolute() {
        Ok(dest.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(dest))
    }
}

fn assert_safe_dest(dest: &Path) -> Result<(), Box<dyn Error>> {
    let cwd = env::current_dir()?;
    if dest == cwd {
        return Err("no voy a sobreescribir el directorio actual".into());
    }

    let home = env::var_os("HOME").map(PathBuf::from);
    let forbidden = [
        PathBuf::from("/"),
        PathBuf::from("/usr"),
        PathBuf::from("/bin"),
        PathBuf::from("/sbin"),
        PathBuf::from("/etc"),
        PathBuf::from("/System"),
        PathBuf::from("/Library"),
        PathBuf::from("/Applications"),
        PathBuf::from("/private"),
    ];
    for p in &forbidden {
        if dest == p {
            return Err(format!("destino prohibido: {}", dest.display()).into());
        }
    }
    if let Some(home) = &home {
        if dest == home {
            return Err("no voy a escribir en $HOME".into());
        }
    }
    Ok(())
}

fn prepare_dest(dest: &Path, force: bool) -> Result<(), Box<dyn Error>> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
        return Ok(());
    }
    if dest.is_file() {
        return Err(format!("{} es un archivo", dest.display()).into());
    }
    let empty = dest.read_dir()?.next().is_none();
    if empty {
        return Ok(());
    }
    if !force {
        return Err(format!(
            "{} ya existe y no está vacío (usa --force si es un replicante anterior)",
            dest.display()
        )
        .into());
    }
    if !looks_like_replicante(dest) {
        return Err(format!(
            "{} no parece un replicante; no lo borro",
            dest.display()
        )
        .into());
    }
    fs::remove_dir_all(dest)?;
    fs::create_dir_all(dest)?;
    Ok(())
}

fn looks_like_replicante(dir: &Path) -> bool {
    let cargo = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
    let main = fs::read_to_string(dir.join("src/main.rs")).unwrap_or_default();
    cargo.contains("name = \"replicante\"")
        && main.contains("const GENERATION:")
        && main.contains("const LINEAGE:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_bumps_generation_and_lineage() {
        let next = rewrite_main(include_str!("main.rs")).unwrap();
        assert!(next.contains(&format!(
            "const GENERATION: u32 = {};",
            GENERATION + 1
        )));
        assert!(next.contains(&format!(
            "const LINEAGE: &str = \"{LINEAGE}.{}\";",
            GENERATION + 1
        )));
        assert!(!next.contains(&format!("const GENERATION: u32 = {GENERATION};")));
    }

    #[test]
    fn genome_lists_the_project() {
        let names: Vec<_> = GENOME.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"src/main.rs"));
        assert!(names.contains(&"Cargo.toml"));
        assert!(names.contains(&"README.md"));
        assert!(names.contains(&"LICENSE"));
        assert!(names.contains(&".gitignore"));
    }

    #[test]
    fn spawn_writes_a_child_project() {
        let dir = env::temp_dir().join(format!("replicante-test-{}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        spawn(&dir, false, false).unwrap();

        let child_main = fs::read_to_string(dir.join("src/main.rs")).unwrap();
        assert!(child_main.contains(&format!(
            "const GENERATION: u32 = {};",
            GENERATION + 1
        )));
        assert!(dir.join("Cargo.toml").exists());
        assert!(dir.join("README.md").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn refuses_to_spawn_over_cwd() {
        let cwd = env::current_dir().unwrap();
        let err = spawn(&cwd, true, false).unwrap_err();
        assert!(err.to_string().contains("directorio actual"));
    }
}
