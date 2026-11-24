mod utils;

use dirs::download_dir;
use std::path::PathBuf;
use utils::filechecker::watch_added_files;
use utils::hashd::{check_file_hash, download_and_load_hashes};

fn get_downloads_folder() -> Option<PathBuf> {
    let path = download_dir()?;
    println!("Downloads folder: {}", path.display());
    Some(path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("initializing hashes");
    let malicious_hashes = download_and_load_hashes()?;
    println!("Loaded {} unique hashes.", malicious_hashes.len());

    let Some(download_fold) = get_downloads_folder() else {
        eprintln!("Could not determine the Downloads folder.");
        return Ok(());
    };

    watch_added_files(&download_fold, true, move |path| {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if matches!(ext, "crdownload" | "part" | "tmp") {
                return;
            }
        }

        println!("New file: {}", path.display());
        match check_file_hash(&path) {
            Ok(hash) => {
                println!("SHA-256: {hash}");
                if malicious_hashes.contains(&hash) {
                    println!("ALERT: Malicious hash detected for {}!", path.display());
                } else {
                    println!("Clean.");
                }
            }
            Err(e) => eprintln!("Failed to hash {}: {e}", path.display()),
        }
    })?;

    Ok(())
}