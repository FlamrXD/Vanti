use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::thread;
use std::time::Duration;

const HASH_URLS: [&str; 3] = [
    "https://raw.githubusercontent.com/amitambekar510/Malicious-Hash-Threat-List/refs/heads/main/hashes/sha256/malicious_SHA256_hashes_aa.txt",
    "https://raw.githubusercontent.com/amitambekar510/Malicious-Hash-Threat-List/refs/heads/main/hashes/sha256/malicious_SHA256_hashes_ab.txt",
    "https://raw.githubusercontent.com/amitambekar510/Malicious-Hash-Threat-List/refs/heads/main/hashes/sha256/malicious_SHA256_hashes_ac.txt",
];

pub fn download_and_load_hashes() -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    fs::create_dir_all("hashes")?;
    let mut hash_set = HashSet::new();

    for (i, url) in HASH_URLS.iter().enumerate() {
        let filename = format!("hashes/malicious_SHA256_hashes_{i}.txt");
        let file_path = Path::new(&filename);

        let needs_download = match fs::metadata(file_path) {
            Ok(meta) => meta.len() == 0,
            Err(_) => true,
        };

        if needs_download {
            println!("Downloading {filename}...");
            let mut response = reqwest::blocking::get(*url)?.error_for_status()?;
            let mut out = File::create(file_path)?;
            io::copy(&mut response, &mut out)?;
        } else {
            println!("Using cached {filename}");
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let clean_hash = line?.trim().to_ascii_lowercase();
            if !clean_hash.is_empty() {
                hash_set.insert(clean_hash);
            }
        }
    }

    Ok(hash_set)
}

pub fn check_file_hash<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let file = retry_open_file(path.as_ref(), 10, Duration::from_millis(500))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn retry_open_file(path: &Path, max_attempts: u32, delay: Duration) -> io::Result<File> {
    let mut attempts = 0;
    loop {
        match File::open(path) {
            Ok(file) => return Ok(file),
            Err(e) => {
                let is_sharing_violation =
                    e.raw_os_error() == Some(32) || e.kind() == io::ErrorKind::PermissionDenied;

                attempts += 1;
                if attempts >= max_attempts || !is_sharing_violation {
                    return Err(e);
                }
                thread::sleep(delay);
            }
        }
    }
}