<div align="center">

<img src="art/icon.png" width="120" height="120" alt="RetroROMs App Icon" />

# 🦀 RetroROMs PC (Rust Port)

![Rust](https://img.shields.io/badge/Language-Rust_2021-DEA584?style=for-the-badge&logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-Windows_10%2F11_x64-0078D6?style=for-the-badge&logo=windows&logoColor=white)
![GUI](https://img.shields.io/badge/GUI-egui_%2B_eframe_0.29-blue?style=for-the-badge)
![Database](https://img.shields.io/badge/Database-SQLite_(rusqlite)-F57C00?style=for-the-badge&logo=sqlite&logoColor=white)
![Build](https://img.shields.io/badge/CI-GitHub_Actions_(windows--latest)-2ea44f?style=for-the-badge&logo=githubactions&logoColor=white)

**Кроссплатформенный нативный ПК-клиент (Windows / Linux) на языке Rust для каталогизации, поиска и загрузки ретро-игр с портала [Emu-Land.net](https://www.emu-land.net) с автоматической раскладкой по папкам консолей, умной распаковкой ZIP-архивов и интеграцией с эмуляторами.**

[Возможности](#-основные-возможности) • [Платформы](#-поддерживаемые-платформы-33) • [Сборка в GitHub Actions](#-сборка-windows-exe-через-github-actions) • [Локальная сборка](#-локальная-сборка-cargo) • [Архитектура](#-архитектура-пк-порта) • [Android-архив](#-android-версия)

</div>

---

## ✨ Основные возможности ПК-версии

* **🖥️ Эргономичный Desktop Layout (16:9, Ultrawide):**
  Полноценный многопанельный интерфейс: вертикальный сайдбар платформ и навигации, верхняя панель с поиском и алфавитом, центральная область каталога с переключением между **Сеткой (Grid)** и плотной **Таблицей (Table)**.
* **📚 Каталог 33 ретро-платформ с реальными играми:**
  Поддержка всех систем с файлами на [Emu-Land.net](https://www.emu-land.net) (Dendy/NES, Sega Genesis, SNES, PS1, N64, GBA, GBC, Game Boy, PC Engine, 3DO, Atari и др.). Платформы без файлов игр исключены.
* **⚡ Динамические категории и алфавитный указатель:**
  Парсинг блока `#pagelist_top` («Топ игр», «Лучшие», «Рейтинг», буквы алфавита, по которым реально есть игры в базе).
* **📦 Выбор ревизий и версий РОМов (`act=getmfl`):**
  Инспектор игры показывает все доступные ревизии: RUS (переводы), USA, EUR, JAP, Hacks, GoodSets. Скачивание любой версии в один клик.
* **⚡ Умная распаковка ZIP-архивов:**
  - **Single-ROM архивы:** автоматически распаковываются в папку консоли, исходный `.zip` удаляется для экономии диска.
  - **Multi-ROM архивы (GoodSet, сборники):** открывают интерактивный диалог с чекбоксами файлов и быстрым фильтром "Только RUS/USA".
* **🎮 Интеграция с эмуляторами на ПК:**
  Возможность настроить пути к любимым эмуляторам (Mesen, Kega Fusion, Snes9x, DuckStation, mGBA, RetroArch) и запускать скачанные игры кнопкой **«▶ Играть в эмуляторе»**.
* **📁 Нативная файловая система:**
  Выбор папки через стандартный системный проводник (`rfd`), кнопка **«📁 Показать в папке»** (`explorer.exe`), мгновенная раскладка по папкам консолей.
* **🎨 5 ретро-тем:**
  Arcade Neon, Cyberpunk 2077, Retrowave Sunset, Classic Dark, GameBoy Matrix.
* **💾 Локальная база данных SQLite:**
  Сохранение порядка и видимости консолей, кэширование, избранное и история загрузок.

---

## 🕹 Поддерживаемые платформы (33)

- **Домашние консоли:** NES / Dendy (`dendy`), Sega Mega Drive (`genesis`), Super Nintendo (`snes`), Sony PlayStation 1 (`psx`), Nintendo 64 (`n64`), Sega 32X (`32x`), Sega CD (`segacd`), Sega Master System (`sms`), Sega SG-1000 (`sg-1000`), PC Engine (`pce`), PC Engine CD (`pcecd`), 3DO (`3do`), Famicom Disk System (`famicom_disk_system`), Neo Geo CD (`neogeocd`), Atari Jaguar (`jaguar`).
- **Портативные системы:** Game Boy Advance (`gba`), Game Boy (`gb`), Game Boy Color (`gbc`), Sega Game Gear (`gg`), Atari Lynx (`lynx`), Neo Geo Pocket (`ngp`), WonderSwan (`ws`), Virtual Boy (`vboy`), Pokémon Mini (`pmini`), Watara Supervision (`sv`).
- **Классические системы:** Atari 2600 (`2600`), Atari 5200 (`5200`), Atari 7800 (`7800`), ColecoVision (`coleco`), Vectrex (`vectrex`), Intellivision (`intellivision`), Emerson Arcadia 2001 (`arcadia`), Fairchild Channel F (`chaf`).

---

## 🚀 Сборка Windows EXE через GitHub Actions

В репозитории настроен автоматический CI workflow: [`.github/workflows/windows-build.yml`](.github/workflows/windows-build.yml).

### Как получить `RetroROMs-PC.exe`:
1. Сделайте `git push` в репозиторий на GitHub.
2. Откройте вкладку **Actions** в репозитории GitHub.
3. Выберите запуск **Build Windows PC Port (Rust)** (или нажмите **Run workflow** для ручного запуска).
4. Процесс соберет проект на нативном раннере `windows-latest` с компилятором MSVC и сохранением кэша зависимостей (`swatinem/rust-cache`).
5. В блоке **Artifacts** скачайте архив `RetroROMs-PC-Windows-x64` с готовым файлом `RetroROMs-PC.exe` и SHA-256 контрольной суммой.

---

## 💻 Локальная сборка (Cargo)

Если на вашем ПК установлен Rust:

```bash
# Клонирование репозитория
git clone https://github.com/your-username/retroms-desktop.git
cd retroms-desktop

# Сборка и запуск приложения
cargo run --release
```

Файл исполняемой программы появится в:
- Windows: `target\release\retroms-desktop.exe`
- Linux: `target/release/retroms-desktop`

---

## 🏗 Архитектура ПК-порта

```
retroms-desktop/
├── .github/workflows/
│   └── windows-build.yml      # CI сборка под Windows с кэшированием
├── src/
│   ├── main.rs                # Главный цикл приложения, eframe runner, шина событий
│   ├── models.rs              # Структуры данных и список 33 консолей
│   ├── config.rs              # Настройки (JSON): папки, темы, пути к эмуляторам
│   ├── db.rs                  # SQLite хранилище (rusqlite + Mutex)
│   ├── scraper.rs             # HTTP парсер Emu-Land (reqwest, scraper, 302 resolver)
│   ├── archive.rs             # ZIP инспектор и распаковщик (zip-rs)
│   ├── downloader.rs          # Фоновый потоковый менеджер загрузок (Tokio)
│   ├── integrations.rs        # Диалоги rfd, запуск эмуляторов, проводник
│   ├── theme.rs               # Ретро палитры и стилизация egui
│   └── ui/                    # Модули интерфейса (сайдбар, тулбар, сетка, таблица, модалки)
├── Cargo.toml                 # Манифест зависимостей Rust
└── android/                   # Исходный код Android-приложения (архив)
```

---

## 📱 Android-версия

Оригинальный исходный код мобильной версии RetroROMs для Android полностью сохранён в каталоге [`android/`](android/) и изолирован от сборки ПК-версии.
