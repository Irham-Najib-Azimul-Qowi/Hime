use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum SelectionError {
    #[error("Failed to access system clipboard: {0}")]
    ClipboardAccess(String),

    #[error("No text was selected or clipboard was empty")]
    NoSelection,

    #[error("Platform selection acquisition error: {0}")]
    PlatformError(String),
}

/// Platform Abstraction Layer for reading currently selected text on the operating system.
///
/// Implements clipboard preservation semantics:
/// If the fallback clipboard method is used, the original clipboard content
/// is restored immediately after capturing the selection.
pub struct SelectionReader;

impl SelectionReader {
    /// Captures the currently selected text.
    ///
    /// On Windows:
    /// Attempts to read selection safely. If simulated copy is used, original clipboard
    /// contents are saved in memory and restored immediately.
    pub fn get_selected_text() -> Result<String, SelectionError> {
        #[cfg(target_os = "windows")]
        {
            Self::get_selected_text_windows()
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(SelectionError::PlatformError(
                "Selected text capture not implemented for non-Windows platforms".to_string(),
            ))
        }
    }

    #[cfg(target_os = "windows")]
    fn get_selected_text_windows() -> Result<String, SelectionError> {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_C,
            VK_CONTROL,
        };

        // 1. Open clipboard and backup current string content
        let backup_text = Self::read_clipboard_text_raw();

        // 2. Synthesize Ctrl+C to trigger copy in focused application
        unsafe {
            let inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_CONTROL,
                            wScan: 0,
                            dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_C,
                            wScan: 0,
                            dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_C,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_CONTROL,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];

            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != 4 {
                warn!("SendInput did not send all 4 key events: {}", sent);
            }
        }

        // Give the target app a brief moment to process the copy
        std::thread::sleep(std::time::Duration::from_millis(50));

        // 3. Read newly captured selection from clipboard
        let captured = Self::read_clipboard_text_raw();

        // 4. Restore original clipboard content if we backed it up
        if let Some(ref original) = backup_text {
            Self::write_clipboard_text_raw(original);
        }

        captured.ok_or(SelectionError::NoSelection)
    }

    #[cfg(target_os = "windows")]
    fn read_clipboard_text_raw() -> Option<String> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
        use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};

        const CF_UNICODETEXT: u32 = 13;

        unsafe {
            if OpenClipboard(HWND::default()).is_err() {
                return None;
            }

            let handle = GetClipboardData(CF_UNICODETEXT);
            let result = if let Ok(h) = handle {
                if !h.0.is_null() {
                    let ptr = GlobalLock(windows::Win32::Foundation::HGLOBAL(h.0)) as *const u16;
                    if !ptr.is_null() {
                        let mut len = 0;
                        while *ptr.add(len) != 0 {
                            len += 1;
                        }
                        let slice = std::slice::from_raw_parts(ptr, len);
                        let s = String::from_utf16_lossy(slice);
                        let _ = GlobalUnlock(windows::Win32::Foundation::HGLOBAL(h.0));
                        Some(s)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let _ = CloseClipboard();
            result
        }
    }

    #[cfg(target_os = "windows")]
    fn write_clipboard_text_raw(text: &str) {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::DataExchange::{
            CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
        };
        use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};

        const CF_UNICODETEXT: u32 = 13;

        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes_len = utf16.len() * std::mem::size_of::<u16>();

        unsafe {
            if OpenClipboard(HWND::default()).is_err() {
                return;
            }

            let _ = EmptyClipboard();

            if let Ok(hglobal) = GlobalAlloc(GHND, bytes_len) {
                let ptr = GlobalLock(hglobal) as *mut u16;
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
                    let _ = GlobalUnlock(hglobal);
                    let _ = SetClipboardData(
                        CF_UNICODETEXT,
                        windows::Win32::Foundation::HANDLE(hglobal.0),
                    );
                }
            }

            let _ = CloseClipboard();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_reader_clipboard_restore() {
        #[cfg(target_os = "windows")]
        {
            let original_test_string = "Hime PAL Clipboard Preservation Test 42";
            SelectionReader::write_clipboard_text_raw(original_test_string);

            let read_back = SelectionReader::read_clipboard_text_raw();
            assert_eq!(read_back.as_deref(), Some(original_test_string));
        }
    }
}
