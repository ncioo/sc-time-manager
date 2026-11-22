use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use anyhow::{anyhow, Result};

use std::{env, io::stderr, path::PathBuf};

use time_manager::app::App;

const STATE_FILE_NAME: &str = "state.json";

fn main() -> Result<()> {
    initialize_panic_handler();
    enable_terminal_colors();

    let current_exe_path = env::current_exe()?;
    let current_dir_path = current_exe_path
        .parent()
        .ok_or_else(|| anyhow!("Could not resolve the executable directory"))?;
    let state_file_path: PathBuf = current_dir_path.join(STATE_FILE_NAME);

    let app = App::init(&state_file_path);

    startup()?;

    let status = app.run();

    shutdown()?;

    status?;

    Ok(())
}

fn startup() -> Result<()> {
    enable_raw_mode()?;
    stderr().execute(EnterAlternateScreen)?;

    Ok(())
}

fn shutdown() -> Result<()> {
    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

pub fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // В хуке не паникуем второй раз: если терминал уже сломан, просто
        // пытаемся вернуть его в нормальное состояние.
        let _ = stderr().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
        original_hook(panic_info);
    }));
}
/// Приложение рисует в stderr, а crossterm проверяет поддержку ANSI по stdout.
/// Если stdout перенаправлен (запуск из скрипта, cmd, конвейера), цвета
/// пропадают — поэтому включаем VT-режим на stderr и подсказываем crossterm,
/// что терминал ANSI поддерживает.
#[cfg(windows)]
fn enable_terminal_colors() {
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_ERROR_HANDLE;
    use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

    // Интерфейс построен на теме, поэтому цвета нужны всегда: иначе переменная
    // NO_COLOR в окружении делает окно белым на чёрном и темы теряют смысл.
    crossterm::style::force_color_output(true);
    time_manager::theme::set_truecolor(true);

    if std::env::var_os("TERM").is_none() {
        std::env::set_var("TERM", "xterm-256color");
    }

    unsafe {
        let handle = GetStdHandle(STD_ERROR_HANDLE);

        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return;
        }

        let mut mode = 0;

        if GetConsoleMode(handle, &mut mode) != 0 {
            SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}

#[cfg(not(windows))]
fn enable_terminal_colors() {
    crossterm::style::force_color_output(true);
    time_manager::theme::set_truecolor(detect_truecolor());

    if std::env::var_os("TERM").is_none() {
        std::env::set_var("TERM", "xterm-256color");
    }
}

/// Terminal.app на macOS понимает 256 цветов, но не 24-битный RGB: на
/// последовательностях `38;2;r;g;b` он показывает мусорные атрибуты.
/// truecolor включаем только там, где терминал его заявляет.
#[cfg(not(windows))]
fn detect_truecolor() -> bool {
    if let Ok(value) = std::env::var("TIME_MANAGER_TRUECOLOR") {
        let value = value.to_lowercase();

        if matches!(value.as_str(), "0" | "false" | "no") {
            return false;
        }

        if matches!(value.as_str(), "1" | "true" | "yes") {
            return true;
        }
    }

    if let Ok(value) = std::env::var("COLORTERM") {
        let value = value.to_lowercase();

        if value.contains("truecolor") || value.contains("24bit") {
            return true;
        }
    }

    if let Ok(value) = std::env::var("TERM") {
        let value = value.to_lowercase();

        if value.contains("truecolor") || value.contains("direct") {
            return true;
        }
    }

    if let Ok(value) = std::env::var("TERM_PROGRAM") {
        let value = value.to_lowercase();

        if matches!(
            value.as_str(),
            "iterm.app" | "wezterm" | "vscode" | "hyper" | "kitty" | "alacritty"
        ) {
            return true;
        }
    }

    false
}
