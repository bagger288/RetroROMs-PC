use crate::models::ZipRomEntry;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

const NON_ROM_EXTS: &[&str] = &[
    "txt", "nfo", "url", "diz", "doc", "pdf", "html", "htm", "png", "jpg", "jpeg", "bmp", "gif",
    "exe", "bat", "cmd", "sh", "lnk", "desktop",
];

pub fn scan_zip_rom_entries(zip_path: &Path) -> io::Result<Vec<ZipRomEntry>> {
    let file = File::open(zip_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let mut entries = Vec::new();

    for i in 0..archive.len() {
        let file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };

        if file.is_dir() {
            continue;
        }

        let name = file.name();
        if name.contains("__MACOSX") || name.ends_with(".DS_Store") {
            continue;
        }

        let ext = Path::new(name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        if NON_ROM_EXTS.contains(&ext.as_str()) {
            continue;
        }

        let display_name = Path::new(name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(name)
            .to_string();

        let size_bytes = file.size();

        entries.push(ZipRomEntry {
            entry_name: name.to_string(),
            display_name,
            size_bytes,
            formatted_size: format_bytes(size_bytes),
            is_selected: true,
        });
    }

    Ok(entries)
}

pub fn extract_single_rom(
    zip_path: &Path,
    target_folder: &Path,
    entry_name: &str,
    delete_zip: bool,
) -> io::Result<PathBuf> {
    fs::create_dir_all(target_folder)?;

    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let mut zip_entry = archive
        .by_name(entry_name)
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, e.to_string()))?;

    let file_name = Path::new(entry_name)
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name in archive"))?;

    let output_path = target_folder.join(file_name);
    let mut outfile = File::create(&output_path)?;
    io::copy(&mut zip_entry, &mut outfile)?;

    drop(zip_entry);
    drop(archive);

    if delete_zip {
        let _ = fs::remove_file(zip_path);
    }

    Ok(output_path)
}

pub fn extract_selected_entries(
    zip_path: &Path,
    selected_entry_names: &[String],
    target_folder: &Path,
    delete_zip: bool,
) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(target_folder)?;

    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let mut extracted = Vec::new();

    for name in selected_entry_names {
        if let Ok(mut zip_entry) = archive.by_name(name) {
            let file_name = Path::new(name).file_name().unwrap_or_else(|| Path::new(name).as_os_str());
            let output_path = target_folder.join(file_name);
            let mut outfile = File::create(&output_path)?;
            io::copy(&mut zip_entry, &mut outfile)?;
            extracted.push(output_path);
        }
    }

    drop(archive);

    if delete_zip && !extracted.is_empty() {
        let _ = fs::remove_file(zip_path);
    }

    Ok(extracted)
}
