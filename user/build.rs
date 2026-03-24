use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

const MODULES: &[(&str, &str)] = &[
    ("cod", "Cod"),
    ("dev", "Dev"),
    ("fa", "Fa"),
    ("fae", "Fae"),
    ("iec", "Iec"),
    ("logos", "Logos"),
    ("md", "Md"),
    ("oct", "Oct"),
    ("ple", "Ple"),
    ("pom", "Pom"),
    ("seti", "Seti"),
    ("weather", "Weather"),
];

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let lock_path = manifest_dir.join("..").join("Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock_path.display());
    let version = lock_version(&lock_path, "nerd-font-symbols")?;
    let crate_dir = find_registry_crate("nerd-font-symbols", &version)?;

    let mut output = String::from("pub const NERD_FONT_SYMBOLS: &[NerdFontSymbol] = &[\n");
    for (module, category) in MODULES {
        let module_path = crate_dir.join("src").join(format!("{module}.rs"));
        println!("cargo:rerun-if-changed={}", module_path.display());
        let contents = fs::read_to_string(&module_path)?;
        for line in contents.lines() {
            let Some(rest) = line.strip_prefix("pub const ") else {
                continue;
            };
            let Some((name, glyph)) = parse_symbol_line(rest) else {
                continue;
            };
            output.push_str(&format!(
                "    NerdFontSymbol {{ name: \"{}\", glyph: \"{}\", category: NerdFontCategory::{} }},\n",
                escape_rust_string(name),
                escape_rust_string(glyph),
                category
            ));
        }
    }
    output.push_str("];\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    fs::write(out_dir.join("nerd_font_data.rs"), output)?;
    Ok(())
}

fn lock_version(lock_path: &Path, crate_name: &str) -> Result<String, Box<dyn Error>> {
    let contents = fs::read_to_string(lock_path)?;
    let mut in_target = false;
    for line in contents.lines() {
        if line.trim() == "[[package]]" {
            in_target = false;
            continue;
        }
        if let Some(name) = line.trim().strip_prefix("name = \"") {
            if name
                .strip_suffix('\"')
                .is_some_and(|value| value == crate_name)
            {
                in_target = true;
            }
            continue;
        }
        if in_target
            && let Some(version) = line.trim().strip_prefix("version = \"")
            && let Some(version) = version.strip_suffix('\"')
        {
            return Ok(version.to_owned());
        }
    }
    Err(format!("failed to find {crate_name} in Cargo.lock").into())
}

fn find_registry_crate(crate_name: &str, version: &str) -> Result<PathBuf, Box<dyn Error>> {
    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join(".cargo")))
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .ok_or("failed to resolve CARGO_HOME")?;
    let registry_src = cargo_home.join("registry").join("src");
    let target_dir = format!("{crate_name}-{version}");
    for entry in fs::read_dir(&registry_src)? {
        let entry = entry?;
        let candidate = entry.path().join(&target_dir);
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "failed to locate {target_dir} under {}",
        registry_src.display()
    )
    .into())
}

fn parse_symbol_line(line: &str) -> Option<(&str, &str)> {
    let (left, right) = line.split_once('=')?;
    let name = left.split_once(':')?.0.trim();
    let glyph = right.trim().trim_end_matches(';').trim();
    let glyph = glyph.strip_prefix('"')?.strip_suffix('"')?;
    Some((name, glyph))
}

fn escape_rust_string(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| character.escape_default())
        .collect()
}
