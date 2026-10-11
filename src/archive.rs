use crate::models::ZipRomEntry;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

    let b = bytes as f64;
    if b < KIB {
        format!("{:.1} B", b)
    } else if b < MIB {
        format!("{:.1} KiB", b / KIB)
    } else if b < GIB {
        format!("{:.1} MiB", b / MIB)
    } else {
        format!("{:.2} GiB", b / GIB)
    }
}

pub fn scan_zip_rom_entries(
    zip_path: &Path,
) -> Result<Vec<ZipRomEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let file = match File::open(zip_path) {
        Ok(f) => f,
        Err(e) => return Err(Box::new(e)),
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => return Err(Box::new(e)),
    };

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let zip_file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };

        if zip_file.is_dir() {
            continue;
        }

        let name = zip_file.name().to_string();
        let lower = name.to_lowercase();

        // Skip metadata / junk files inside archives
        if lower.contains("__macosx")
            || lower.ends_with(".ds_store")
            || lower.ends_with("thumbs.db")
            || lower.ends_with(".txt")
            || lower.ends_with(".nfo")
            || lower.ends_with(".url")
        {
            continue;
        }

        let file_size = zip_file.size();
        let display_name = Path::new(&name)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| name.clone());

        entries.push(ZipRomEntry {
            entry_name: name,
            display_name,
            size_bytes: file_size,
            formatted_size: format_bytes(file_size),
            is_selected: true,
        });
    }

    Ok(entries)
}

pub fn extract_single_rom(
    zip_path: &Path,
    target_dir: &Path,
    entry_name: &str,
    delete_zip: bool,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut zip_file = archive.by_name(entry_name)?;

    fs::create_dir_all(target_dir)?;

    let file_name = Path::new(entry_name)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "game.rom".to_string());

    let out_path = target_dir.join(&file_name);
    let mut outfile = File::create(&out_path)?;
    io::copy(&mut zip_file, &mut outfile)?;

    drop(zip_file);
    drop(archive);

    if delete_zip {
        let _ = fs::remove_file(zip_path);
    }

    Ok(out_path)
}

pub fn extract_selected_entries(
    zip_path: &Path,
    selected_entries: &[String],
    target_dir: &Path,
    delete_zip: bool,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    fs::create_dir_all(target_dir)?;

    let mut extracted = Vec::new();
    for entry_name in selected_entries {
        if let Ok(mut zip_file) = archive.by_name(entry_name) {
            let file_name = Path::new(entry_name)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| entry_name.clone());

            let out_path = target_dir.join(&file_name);
            if let Ok(mut outfile) = File::create(&out_path) {
                if io::copy(&mut zip_file, &mut outfile).is_ok() {
                    extracted.push(out_path);
                }
            }
        }
    }

    drop(archive);

    if delete_zip && !extracted.is_empty() {
        let _ = fs::remove_file(zip_path);
    }

    Ok(extracted)
}
