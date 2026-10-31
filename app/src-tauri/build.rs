fn main() {
    // Rebuild when the workflow changes the injected credentials.
    println!("cargo:rerun-if-env-changed=VELMORA_TMDB_READ_TOKEN");
    println!("cargo:rerun-if-env-changed=VELMORA_TMDB_API_KEY");
    println!("cargo:rerun-if-env-changed=VELMORA_ANILIST_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=VELMORA_TRAKT_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=VELMORA_TRAKT_CLIENT_SECRET");
    tauri_build::build()
}
