//! Чтение и запись системного буфера обмена.

/// Возвращает текст из буфера обмена. Пустая строка или ошибка — None.
pub fn read_clipboard() -> Option<String> {
    read_clipboard_impl().filter(|text| !text.is_empty())
}

/// Кладёт текст в буфер обмена.
pub fn write_clipboard(text: &str) -> Result<(), String> {
    write_clipboard_impl(text)
}

#[cfg(target_os = "windows")]
fn read_clipboard_impl() -> Option<String> {
    use std::ptr;
    use winapi::shared::minwindef::HGLOBAL;
    use winapi::um::winbase::{GlobalLock, GlobalUnlock};
    use winapi::um::winuser::{
        CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard, CF_UNICODETEXT,
    };

    unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT) == 0 {
            return None;
        }

        if OpenClipboard(ptr::null_mut()) == 0 {
            return None;
        }

        let handle = GetClipboardData(CF_UNICODETEXT);
        let mut text = None;

        if !handle.is_null() {
            let pointer = GlobalLock(handle as HGLOBAL) as *const u16;

            if !pointer.is_null() {
                let mut length = 0usize;

                while *pointer.add(length) != 0 {
                    length += 1;
                }

                text = Some(String::from_utf16_lossy(std::slice::from_raw_parts(
                    pointer, length,
                )));

                GlobalUnlock(handle as HGLOBAL);
            }
        }

        CloseClipboard();

        text
    }
}

#[cfg(target_os = "macos")]
fn read_clipboard_impl() -> Option<String> {
    read_from_command("pbpaste", &[])
}

#[cfg(all(unix, not(target_os = "macos")))]
fn read_clipboard_impl() -> Option<String> {
    read_from_command("xclip", &["-selection", "clipboard", "-o"])
        .or_else(|| read_from_command("xsel", &["--clipboard", "--output"]))
}

#[cfg(not(target_os = "windows"))]
fn read_from_command(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout).ok()
}

#[cfg(target_os = "windows")]
fn write_clipboard_impl(text: &str) -> Result<(), String> {
    use std::ptr;
    use winapi::shared::minwindef::HGLOBAL;
    use winapi::um::winbase::{GlobalAlloc, GlobalFree, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use winapi::um::winuser::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData, CF_UNICODETEXT,
    };

    unsafe {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        if OpenClipboard(ptr::null_mut()) == 0 {
            return Err("could not open the clipboard".to_string());
        }

        let result = (|| {
            if EmptyClipboard() == 0 {
                return Err("could not clear the clipboard".to_string());
            }

            let handle: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2);

            if handle.is_null() {
                return Err("could not allocate clipboard memory".to_string());
            }

            let pointer = GlobalLock(handle);

            if pointer.is_null() {
                GlobalFree(handle);
                return Err("could not lock clipboard memory".to_string());
            }

            std::ptr::copy_nonoverlapping(wide.as_ptr(), pointer as *mut u16, wide.len());
            GlobalUnlock(handle);

            if SetClipboardData(CF_UNICODETEXT, handle).is_null() {
                GlobalFree(handle);
                return Err("could not set clipboard data".to_string());
            }

            Ok(())
        })();

        CloseClipboard();

        result
    }
}

#[cfg(target_os = "macos")]
fn write_clipboard_impl(text: &str) -> Result<(), String> {
    write_to_command("pbcopy", &[], text)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn write_clipboard_impl(text: &str) -> Result<(), String> {
    write_to_command("xclip", &["-selection", "clipboard"], text)
        .or_else(|_| write_to_command("xsel", &["--clipboard", "--input"], text))
}

#[cfg(not(target_os = "windows"))]
fn write_to_command(program: &str, args: &[&str], text: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("{program}: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|error| format!("{program}: {error}"))?;
    }

    let status = child
        .wait()
        .map_err(|error| format!("{program}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited with {status}"))
    }
}
