use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Deserialize)]
pub struct Resource {
    pub file: String,
    pub source: String,
    pub sha256: String,
    pub bytes: u64,
}
pub fn directory() -> PathBuf {
    if let Some(path) = std::env::var_os("PKS_DANISH_RESOURCES") {
        return path.into();
    }
    directories::ProjectDirs::from("", "", "pks-agent-danish")
        .expect("home directory is required")
        .data_local_dir()
        .join("resources")
}
pub fn records() -> Vec<Resource> {
    serde_json::from_str(include_str!("../resources.lock.json"))
        .expect("embedded resource manifest")
}
pub fn verify(path: &Path, record: &Resource) -> bool {
    let Ok(data) = fs::read(path) else {
        return false;
    };
    data.len() as u64 == record.bytes && format!("{:x}", Sha256::digest(&data)) == record.sha256
}
pub fn setup(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)?;
    for r in records() {
        let target = dir.join(&r.file);
        if verify(&target, &r) {
            eprintln!("OK {}", r.file);
            continue;
        }
        let mut response = ureq::get(&r.source)
            .call()
            .with_context(|| format!("Download {}", r.file))?;
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(r.bytes + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != r.bytes || format!("{:x}", Sha256::digest(&bytes)) != r.sha256 {
            bail!("Forkert størrelse eller SHA-256: {}", r.file);
        }
        let tmp = target.with_extension(format!("download-{}", std::process::id()));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        // Windows does not replace an existing destination with rename.
        if cfg!(windows) && target.exists() {
            fs::remove_file(&target)?;
        }
        fs::rename(&tmp, &target)?;
        eprintln!("Installeret {}", r.file);
    }
    eprintln!("Sprogdata klar: {}", dir.display());
    Ok(())
}
