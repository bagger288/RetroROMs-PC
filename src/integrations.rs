use std::path::{Path, PathBuf};
use std::process::Command;

pub fn pick_download_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Выберите папку для сохранения ROMs")
        .pick_folder()
}

pub fn pick_executable_file(title: &str) -> Option<PathBuf> {
    let dialog_title = if title.is_empty() {
        "Выберите исполняемый файл эмулятора"
    } else {
        title
    };
    rfd::FileDialog::new()
        .set_title(dialog_title)
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

pub fn launch_retroarch(
    retroarch_exe: &str,
    rom_path: &Path,
    core_name_or_path: Option<&str>,
    extra_args: &str,
) -> std::io::Result<()> {
    let mut cmd = Command::new(retroarch_exe);

    if let Some(core) = core_name_or_path {
        let trimmed_core = core.trim();
        if !trimmed_core.is_empty() {
            cmd.arg("-L");
            let core_path = PathBuf::from(trimmed_core);
            if core_path.exists() {
                cmd.arg(core_path);
            } else {
                let exe_dir = Path::new(retroarch_exe).parent().unwrap_or_else(|| Path::new(""));
                let candidate1 = exe_dir.join("cores").join(trimmed_core);
                let candidate2 = exe_dir.join("cores").join(format!("{}.dll", trimmed_core));
                let candidate3 = exe_dir.join("cores").join(format!("{}_libretro.dll", trimmed_core));

                if candidate1.exists() {
                    cmd.arg(candidate1);
                } else if candidate2.exists() {
                    cmd.arg(candidate2);
                } else if candidate3.exists() {
                    cmd.arg(candidate3);
                } else {
                    cmd.arg(trimmed_core);
                }
            }
        }
    }

    if !extra_args.trim().is_empty() {
        for arg in extra_args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    cmd.arg(rom_path);
    cmd.spawn()?;
    Ok(())
}
