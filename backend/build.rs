use dotenv::dotenv;
use ic_cdk_bindgen::{Builder, Config};
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

fn generate(src_path: impl AsRef<Path>, dst_path: impl AsRef<Path>) -> io::Result<()> {
    let src = BufReader::new(fs::File::open(src_path.as_ref())?);
    let mut dst = BufWriter::new(fs::File::create(dst_path.as_ref())?);

    writeln!(dst, "[")?;
    for word in src.lines() {
        writeln!(dst, "\"{}\",", &word.unwrap())?;
    }
    writeln!(dst, "]")
}

fn main() -> Result<(), String> {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let adjectives_path = Path::new(&out_dir).join("adjectives.rs");
    generate("data/adjectives.txt", &adjectives_path).map_err(|err| {
        format!(
            "failed to create list of adjectives from {}: {}",
            adjectives_path.display(),
            err
        )
    })?;
    let nouns_path = Path::new(&out_dir).join("nouns.rs");
    generate("data/nouns.txt", &nouns_path).map_err(|err| {
        format!(
            "failed to create list of nouns from {}: {}",
            nouns_path.display(),
            err
        )
    })?;

    // VETKD system API integration
    dotenv().ok();

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));

    let vetkd_system_api_did_path =
        manifest_dir.join("../src/declarations/vetkd_system_api/vetkd_system_api.did");
    let vetkd_system_api_did_str = vetkd_system_api_did_path.to_str().expect("Path invalid");

    unsafe {
        env::set_var(
            "CANISTER_CANDID_PATH_VETKD_SYSTEM_API",
            vetkd_system_api_did_str,
        );
    }

    // vetkd_system_api configuration
    let mut vetkd_system_api = Config::new("vetkd_system_api");
    vetkd_system_api
        .binding
        .set_type_attributes("#[derive(Debug, CandidType, Deserialize)]".into());

    let mut builder = Builder::new();
    builder.add(vetkd_system_api);
    builder.build(Some(manifest_dir.join("src/declarations")));

    Ok(())
}
