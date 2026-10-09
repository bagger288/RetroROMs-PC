use std::path::{Path, PathBuf};
use std::process::Command;

pub fn pick_download_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Выберите папку для сохранения ROM-файлов")
        .pick_folder()
}

pub fn pick_executable_file(title: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(title)
        .pick_file()
}

pub fn reveal_in_file_explorer(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    {
        if path.is_file() {
            Command::new("explorer")
                .arg(format!("/select,\"{}\"", path.display()))
                .spawn()?;
        } else {
            Command::new("explorer")
                .arg(path)
                .spawn()?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if path.is_file() {
            Command::new("open")
                .arg("-R")
                .arg(path)
                .spawn()?;
        } else {
            Command::new("open")
                .arg(path)
                .spawn()?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        let target = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        Command::new("xdg-open")
            .arg(target)
            .spawn()?;
    }

    Ok(())
}

pub fn launch_emulator(
    emulator_exe: &str,
    rom_path: &Path,
    custom_args: Option<&str>,
) -> Result<(), std::io::Error> {
    let mut cmd = Command::new(emulator_exe);

    if let Some(args_str) = custom_args {
        if !args_str.trim().is_empty() {
            for arg in args_str.split_whitespace() {
                cmd.arg(arg);
            }
        }
    }

    cmd.arg(rom_path);
    cmd.spawn()?;
    Ok(())
}
