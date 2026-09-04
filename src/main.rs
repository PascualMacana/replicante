//! Constructor auto-reproductor.
//!
//! El binario lleva su genoma (fuentes) embebido en compile-time.
//! `spawn <dir>` escribe un proyecto Cargo hijo que, al compilares, puede hacer lo mismo.

mod dish;

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
    ("src/dish.rs", include_str!("dish.rs")),
    ("cell.svg", include_str!("../cell.svg")),
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
            if let Err(e) = spawn_cmd(Path::new(&dest), force, build) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("dish") => {
            let mut splits: u32 = 8;
            let mut delay_ms: u64 = 80;
            while let Some(flag) = args.next() {
                match flag.as_str() {
                    "--gens" | "--splits" => {
                        let v = args.next().unwrap_or_else(|| {
                            eprintln!("uso: replicante dish --gens N");
                            process::exit(2);
                        });
                        splits = v.parse().unwrap_or_else(|_| {
                            eprintln!("no es un número: {v}");
                            process::exit(2);
                        });
                    }
                    "--delay" => {
                        let v = args.next().unwrap_or_else(|| {
                            eprintln!("uso: replicante dish --delay MS");
                            process::exit(2);
                        });
                        delay_ms = v.parse().unwrap_or_else(|_| {
                            eprintln!("no es un número: {v}");
                            process::exit(2);
                        });
                    }
                    other => {
                        eprintln!("flag desconocida: {other}");
                        process::exit(2);
                    }
                }
            }
            if let Err(e) = dish::run(GENERATION, LINEAGE, splits, delay_ms) {
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

  replicante identity          generación, linaje, hijas, archivos del genoma
  replicante genome            imprime las fuentes embebidas
  replicante dish              anima una célula brotando hijas
                   --gens N    cuántas hijas (default 8)
                   --delay MS  ms entre frames (default 80)
  replicante spawn <dir>       escribe un proyecto Cargo hijo
                   --build     compila al hijo con cargo
                   --force     pisa un hijo anterior

El linaje cuenta hijas, no generaciones: 0 → 0.1 → 0.1.1.
Un segundo hijo de 0 es 0.2. No se copia por la red, no pisa el
directorio actual, un solo hijo por corrida."
    );
}

fn identity() {
    let buds = read_brotes(Path::new("."));
    println!("replicante");
    println!("generación  {GENERATION}");
    println!("linaje      {LINEAGE}");
    println!("hijas       {buds}");
    println!("próximo     {}", child_lineage(LINEAGE, buds));
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

const BROTES: &str = ".brotes";

pub(crate) fn child_lineage(parent: &str, buds: u32) -> String {
    format!("{parent}.{}", buds + 1)
}

fn spawn_cmd(dest: &Path, force: bool, build: bool) -> Result<(), Box<dyn Error>> {
    let dest = normalize_dest(dest)?;
    let (lineage, record) = plan_birth(Path::new("."), LINEAGE, &dest, force)?;
    spawn_lineage(&dest, force, build, &lineage)?;
    if record {
        record_birth(Path::new("."))?;
    }
    Ok(())
}

#[cfg(test)]
fn spawn(dest: &Path, force: bool, build: bool) -> Result<(), Box<dyn Error>> {
    spawn_lineage(dest, force, build, &child_lineage(LINEAGE, 0))
}

fn spawn_lineage(
    dest: &Path,
    force: bool,
    build: bool,
    next_lineage: &str,
) -> Result<(), Box<dyn Error>> {
    let dest = normalize_dest(dest)?;
    assert_safe_dest(&dest)?;
    prepare_dest(&dest, force)?;

    let next_gen = GENERATION + 1;
    let child_main = rewrite_main(include_str!("main.rs"), next_gen, next_lineage)?;

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

fn rewrite_main(src: &str, generation: u32, lineage: &str) -> Result<String, Box<dyn Error>> {
    let src = patch_const_u32(src, "GENERATION", generation)?;
    patch_const_str(&src, "LINEAGE", lineage)
}

fn patch_const_u32(src: &str, name: &str, new: u32) -> Result<String, Box<dyn Error>> {
    let start_pat = format!("const {name}: u32 = ");
    let start = src
        .find(&start_pat)
        .ok_or_else(|| format!("no encuentro {name} en el genoma"))?;
    let value_start = start + start_pat.len();
    let rel_end = src[value_start..]
        .find(';')
        .ok_or_else(|| format!("const {name} sin cierre"))?;
    let mut out = String::with_capacity(src.len() + 8);
    out.push_str(&src[..value_start]);
    out.push_str(&new.to_string());
    out.push_str(&src[value_start + rel_end..]);
    Ok(out)
}

fn patch_const_str(src: &str, name: &str, new_val: &str) -> Result<String, Box<dyn Error>> {
    if new_val.contains('"') || new_val.contains('\\') {
        return Err("el valor no puede tener comillas ni backslash".into());
    }
    let start_pat = format!("const {name}: &str = \"");
    let start = src
        .find(&start_pat)
        .ok_or_else(|| format!("no encuentro {name} en el genoma"))?;
    let value_start = start + start_pat.len();
    let rel_end = src[value_start..]
        .find('"')
        .ok_or_else(|| format!("const {name} sin cierre"))?;
    let mut out = String::with_capacity(src.len() + new_val.len());
    out.push_str(&src[..value_start]);
    out.push_str(new_val);
    out.push_str(&src[value_start + rel_end..]);
    Ok(out)
}

fn read_const_str(dir: &Path, name: &str) -> Option<String> {
    let src = fs::read_to_string(dir.join("src/main.rs")).ok()?;
    let start_pat = format!("const {name}: &str = \"");
    let start = src.find(&start_pat)?;
    let value_start = start + start_pat.len();
    let rel_end = src[value_start..].find('"')?;
    Some(src[value_start..value_start + rel_end].to_string())
}

fn read_brotes(dir: &Path) -> u32 {
    fs::read_to_string(dir.join(BROTES))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn record_birth(parent: &Path) -> Result<(), Box<dyn Error>> {
    if !looks_like_replicante(parent) {
        return Ok(());
    }
    let n = read_brotes(parent) + 1;
    fs::write(parent.join(BROTES), format!("{n}\n"))?;
    println!(
        "padre  hijas {n}  → próximo linaje {}",
        child_lineage(LINEAGE, n)
    );
    Ok(())
}

fn plan_birth(
    parent: &Path,
    parent_lin: &str,
    dest: &Path,
    force: bool,
) -> Result<(String, bool), Box<dyn Error>> {
    if force && dest.exists() && looks_like_replicante(dest) {
        if let Some(lin) = read_const_str(dest, "LINEAGE") {
            return Ok((lin, false));
        }
    }
    Ok((child_lineage(parent_lin, read_brotes(parent)), true))
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
        return Err(format!("{} no parece un replicante; no lo borro", dest.display()).into());
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
    fn lineage_counts_daughters_not_generations() {
        assert_eq!(child_lineage("0", 0), "0.1");
        assert_eq!(child_lineage("0", 1), "0.2");
        assert_eq!(child_lineage("0.1", 0), "0.1.1");
        assert_ne!(child_lineage("0.1", 0), "0.1.2");
    }

    #[test]
    fn rewrite_bumps_generation_and_lineage() {
        let next = rewrite_main(include_str!("main.rs"), 1, "0.1").unwrap();
        assert!(next.contains("const GENERATION: u32 = 1;"));
        assert!(next.contains("const LINEAGE: &str = \"0.1\";"));
    }

    #[test]
    fn grandchild_lineage_is_not_generation() {
        let child = rewrite_main(include_str!("main.rs"), 1, &child_lineage(LINEAGE, 0)).unwrap();
        let grand = rewrite_main(&child, 2, &child_lineage("0.1", 0)).unwrap();
        assert!(grand.contains("const LINEAGE: &str = \"0.1.1\";"));
        assert!(!grand.contains("const LINEAGE: &str = \"0.1.2\";"));
    }

    #[test]
    fn genome_lists_the_project() {
        let names: Vec<_> = GENOME.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"src/main.rs"));
        assert!(names.contains(&"src/dish.rs"));
        assert!(names.contains(&"cell.svg"));
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
        assert!(child_main.contains(&format!("const GENERATION: u32 = {};", GENERATION + 1)));
        assert!(child_main.contains("const LINEAGE: &str = \"0.1\";"));
        assert!(dir.join("Cargo.toml").exists());
        assert!(dir.join("README.md").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn brotes_count_siblings() {
        let dir = env::temp_dir().join(format!("replicante-brotes-{}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("Cargo.toml"), "name = \"replicante\"\n").unwrap();
        fs::write(
            dir.join("src/main.rs"),
            "const GENERATION: u32 = 0;\nconst LINEAGE: &str = \"0\";\n",
        )
        .unwrap();
        assert_eq!(read_brotes(&dir), 0);
        record_birth(&dir).unwrap();
        assert_eq!(read_brotes(&dir), 1);
        record_birth(&dir).unwrap();
        assert_eq!(read_brotes(&dir), 2);
        assert_eq!(child_lineage("0", read_brotes(&dir)), "0.3");
        let (lin, rec) = plan_birth(&dir, "0", &dir.join("no-existe"), false).unwrap();
        assert_eq!(lin, "0.3");
        assert!(rec);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn refuses_to_spawn_over_cwd() {
        let cwd = env::current_dir().unwrap();
        let err = spawn(&cwd, true, false).unwrap_err();
        assert!(err.to_string().contains("directorio actual"));
    }
}
