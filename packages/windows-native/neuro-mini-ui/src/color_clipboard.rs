//! 仅显式复制操作写剪贴板；发布前准备全部 UTF-16 数据，失败保留 Win32 错误并释放内存。
use windows::{
    core::{Error, Result},
    Win32::{
        Foundation::*,
        System::{DataExchange::*, Memory::*},
        UI::WindowsAndMessaging::IsWindow,
    },
};
struct Clipboard;
impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}
struct Memory(Option<HGLOBAL>);
impl Drop for Memory {
    fn drop(&mut self) {
        if let Some(h) = self.0 {
            unsafe {
                let _ = GlobalFree(Some(h));
            }
        }
    }
}
pub(super) unsafe fn copy(owner: HWND, text: &str) -> Result<()> {
    let units: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let handle = GlobalAlloc(GMEM_MOVEABLE, units.len() * 2)?;
    let mut memory = Memory(Some(handle));
    let pointer = GlobalLock(handle);
    if pointer.is_null() {
        return Err(Error::from_win32());
    }
    std::ptr::copy_nonoverlapping(units.as_ptr(), pointer.cast(), units.len());
    let _ = GlobalUnlock(handle);
    OpenClipboard(Some(owner))?;
    let _clipboard = Clipboard;
    if !IsWindow(Some(owner)).as_bool() {
        return Err(Error::new(E_HANDLE, "Color picker closed before copying"));
    }
    EmptyClipboard()?;
    SetClipboardData(13, Some(HANDLE(handle.0)))?;
    memory.0 = None;
    Ok(())
}
