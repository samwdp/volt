mod build_output;

use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use build_output::{distributed_user_library_paths, install_root_library_link};

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
    let symbols_dir = manifest_dir.join("nerd_font_symbols");

    let mut output = String::from("pub const ICON_FONT_SYMBOLS: &[IconFontSymbol] = &[\n");
    for (module, category) in MODULES {
        let module_path = symbols_dir.join(format!("{module}.rs"));
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
                "    IconFontSymbol {{ name: \"{}\", glyph: \"{}\", category: IconFontCategory::{} }},\n",
                escape_rust_string(name),
                escape_rust_string(glyph),
                category
            ));
        }
    }
    output.push_str("];\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    fs::write(out_dir.join("icon_font_data.rs"), output)?;
    link_root_user_library(&manifest_dir, &out_dir)?;
    Ok(())
}

fn link_root_user_library(manifest_dir: &Path, out_dir: &Path) -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");
    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let Some(paths) = distributed_user_library_paths(manifest_dir, out_dir, &target_os) else {
        return Ok(());
    };

    if let Err(error) = install_root_library_link(&paths) {
        println!(
            "cargo:warning=failed to expose `{}` at repository root `{}`: {error}",
            paths.built_library_path.display(),
            paths.root_library_path.display()
        );
    }

    Ok(())
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
