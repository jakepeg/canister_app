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
    builder.build(Some(manifest_dir.join("src/declarations"))); // Added ? for error propagation

    // --- Embed Backend Wasm ---
    println!("cargo:rerun-if-changed=build.rs"); // Rerun if build script changes

    // Determine the expected path of the Wasm file
    // Assumes a standard release build structure. Adjust if needed.
    let wasm_path = manifest_dir.join("../../target/wasm32-unknown-unknown/release/backend.wasm");

    println!("cargo:rerun-if-changed={}", wasm_path.display()); // Rerun if Wasm changes

    if wasm_path.exists() {
        let wasm_bytes = fs::read(&wasm_path).map_err(|e| {
            format!(
                "Failed to read backend Wasm file at {}: {}",
                wasm_path.display(),
                e
            )
        })?;

        let out_path = Path::new(&out_dir).join("backend_wasm.rs");
        let mut writer = BufWriter::new(fs::File::create(&out_path).map_err(|e| {
            format!(
                "Failed to create backend_wasm.rs at {}: {}",
                out_path.display(),
                e
            )
        })?);

        // Write the byte array definition to the file
        writeln!(writer, "pub const BACKEND_WASM: &[u8] = &{:?};", wasm_bytes)
            .map_err(|e| format!("Failed to write Wasm bytes to backend_wasm.rs: {}", e))?;

        println!(
            "Embedded backend Wasm ({} bytes) into {}",
            wasm_bytes.len(),
            out_path.display()
        );
    } else {
        // If the Wasm file doesn't exist (e.g., during initial `cargo check`),
        // create an empty placeholder to avoid compilation errors.
        // The actual build process (`dfx build`) should ensure the Wasm exists later.
        println!(
            "cargo:warning=Backend Wasm file not found at {}. Creating empty placeholder.",
            wasm_path.display()
        );
        let out_path = Path::new(&out_dir).join("backend_wasm.rs");
        let mut writer = BufWriter::new(fs::File::create(&out_path).map_err(|e| {
            format!(
                "Failed to create empty backend_wasm.rs at {}: {}",
                out_path.display(),
                e
            )
        })?);
        writeln!(writer, "pub const BACKEND_WASM: &[u8] = &[];")
            .map_err(|e| format!("Failed to write empty Wasm bytes: {}", e))?;
    }
    // --- End Embed Backend Wasm ---

    Ok(())
}
