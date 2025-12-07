use std::fs;
use std::path::Path;

fn main() {
    // Ensure the frontend/dist directory exists to satisfy RustEmbed
    let dist_dir = Path::new("frontend/dist");
    if !dist_dir.exists() {
        // Create the directory
        if let Err(e) = fs::create_dir_all(dist_dir) {
            println!("cargo:warning=Failed to create frontend/dist: {}", e);
        } else {
             // Create a dummy index.html so the fallback doesn't crash runtime if accessed
             let index_path = dist_dir.join("index.html");
             if !index_path.exists() {
                 let _ = fs::write(index_path, "<html><body>Frontend not built. Run 'npm run build' in llmspell-web/frontend</body></html>");
             }
        }
    }

    println!("cargo:rerun-if-changed=frontend/dist");
}
