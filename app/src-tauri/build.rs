fn main() {
    // Rebuild when the release workflow changes the injected TMDB credentials.
    println!("cargo:rerun-if-env-changed=VELMORA_TMDB_READ_TOKEN");
    println!("cargo:rerun-if-env-changed=VELMORA_TMDB_API_KEY");
    tauri_build::build()
}
