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
    let clean_exe = emulator_exe.trim().trim_matches('"').trim_matches('\'');
    println!("[EMULATOR] Запуск эмулятора: {:?}", clean_exe);
    println!("[EMULATOR] ROM файл: {:?}", rom_path);

    let exe_path = PathBuf::from(clean_exe);
    if !exe_path.exists() {
        let err_msg = format!("Исполняемый файл эмулятора не найден: {}", clean_exe);
        println!("[EMULATOR ОШИБКА] {}", err_msg);
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, err_msg));
    }

    let mut cmd = Command::new(&exe_path);
    if let Some(exe_dir) = exe_path.parent() {
        if exe_dir.exists() {
            cmd.current_dir(exe_dir);
        }
    }

    if !args.trim().is_empty() {
        for arg in args.split_whitespace() {
            cmd.arg(arg);
        }
    }
    cmd.arg(rom_path);

    println!("[EMULATOR] Выполняемая команда: {:?}", cmd);
    match cmd.spawn() {
        Ok(child) => {
            println!("[EMULATOR] Успешно запущен! Процесс PID: {}", child.id());
            Ok(())
        }
        Err(e) => {
            println!("[EMULATOR ОШИБКА] Не удалось выполнить spawn(): {}", e);
            Err(e)
        }
    }
}

pub fn launch_retroarch(
    retroarch_exe: &str,
    rom_path: &Path,
    core_name_or_path: Option<&str>,
    extra_args: &str,
) -> std::io::Result<()> {
    let clean_exe = retroarch_exe.trim().trim_matches('"').trim_matches('\'');
    println!("--------------------------------------------------");
    println!("[RETROARCH] Запуск RetroArch: {}", clean_exe);
    println!("[RETROARCH] ROM файл: {}", rom_path.display());

    let exe_path = PathBuf::from(clean_exe);
    if !exe_path.exists() {
        let err_msg = format!("Файл RetroArch не существует по пути: {}", clean_exe);
        println!("[RETROARCH ОШИБКА] {}", err_msg);
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, err_msg));
    }

    let mut cmd = Command::new(&exe_path);

    // CRITICAL: Set working directory to RetroArch directory so it finds retroarch.cfg and cores/
    if let Some(exe_dir) = exe_path.parent() {
        if exe_dir.exists() {
            println!("[RETROARCH] Рабочая директория: {}", exe_dir.display());
            cmd.current_dir(exe_dir);
        }
    }

    let mut core_applied = false;
    let mut chosen_core_display = String::new();

    if let Some(core) = core_name_or_path {
        let trimmed_core = core.trim().trim_matches('"').trim_matches('\'');
        if !trimmed_core.is_empty() {
            let core_path = PathBuf::from(trimmed_core);
            if core_path.exists() {
                println!("[RETROARCH] Путь к ядру: {}", core_path.display());
                cmd.arg("-L");
                cmd.arg(&core_path);
                chosen_core_display = core_path.display().to_string();
                core_applied = true;
            } else {
                let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new(""));
                let candidate1 = exe_dir.join("cores").join(trimmed_core);
                let candidate2 = exe_dir.join("cores").join(format!("{}.dll", trimmed_core));
                let candidate3 = exe_dir.join("cores").join(format!("{}_libretro.dll", trimmed_core));

                if candidate1.exists() {
                    println!("[RETROARCH] Найдено ядро в cores/: {}", candidate1.display());
                    cmd.arg("-L");
                    cmd.arg(&candidate1);
                    chosen_core_display = candidate1.display().to_string();
                    core_applied = true;
                } else if candidate2.exists() {
                    println!("[RETROARCH] Найдено ядро в cores/: {}", candidate2.display());
                    cmd.arg("-L");
                    cmd.arg(&candidate2);
                    chosen_core_display = candidate2.display().to_string();
                    core_applied = true;
                } else if candidate3.exists() {
                    println!("[RETROARCH] Найдено ядро в cores/: {}", candidate3.display());
                    cmd.arg("-L");
                    cmd.arg(&candidate3);
                    chosen_core_display = candidate3.display().to_string();
                    core_applied = true;
                } else {
                    println!("[RETROARCH] Имя ядра для RetroArch: {}", trimmed_core);
                    cmd.arg("-L");
                    cmd.arg(trimmed_core);
                    chosen_core_display = trimmed_core.to_string();
                    core_applied = true;
                }
            }
        }
    }

    if !core_applied {
        println!("[RETROARCH] Ядро не указано, RetroArch выберет его автоматически.");
    }

    if !extra_args.trim().is_empty() {
        for arg in extra_args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    cmd.arg(rom_path);

    if core_applied {
        println!("[RETROARCH] Команда запуска: \"{}\" -L \"{}\" \"{}\"", clean_exe, chosen_core_display, rom_path.display());
    } else {
        println!("[RETROARCH] Команда запуска: \"{}\" \"{}\"", clean_exe, rom_path.display());
    }
    match cmd.spawn() {
        Ok(child) => {
            println!("[RETROARCH] Процесс успешно создан! PID: {}", child.id());
            println!("--------------------------------------------------");
            Ok(())
        }
        Err(e) => {
            println!("[RETROARCH ОШИБКА] Ошибка при spawn(): {}", e);
            println!("--------------------------------------------------");
            Err(e)
        }
    }
}
