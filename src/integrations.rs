use std::path::{Path, PathBuf};
use std::process::Command;

pub fn pick_download_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Выберите папку для сохранения ROMs")
        .pick_folder()
}

pub fn pick_executable_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Выберите исполняемый файл эмулятора")
        .pick_file()
}

pub fn reveal_in_file_explorer(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()?;
        Ok(())
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        if let Some(parent) = path.parent() {
            let _ = open::that(parent);
        } else {
            let _ = open::that(path);
        }
        Ok(())
    }
}

pub fn launch_emulator(emulator_exe: &str, rom_path: &Path, args: &str) -> std::io::Result<()> {
    let mut cmd = Command::new(emulator_exe);
    if !args.trim().is_empty() {
        for arg in args.split_whitespace() {
            cmd.arg(arg);
        }
    }
    cmd.arg(rom_path);
    cmd.spawn()?;
    Ok(())
}
