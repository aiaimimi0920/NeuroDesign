//! 子 HWND 焦点滚入与滚轮转发；每个 subclass 随对应子窗口同步卸载。
use super::*;
use windows::Win32::UI::Shell::*;
const ID: usize = 0x4e53;
pub(super) unsafe fn install(hwnd: HWND) -> Result<()> {
    SetWindowSubclass(hwnd, Some(procedure), ID, 0).ok()
}
unsafe extern "system" fn procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    if let Ok(parent) = GetParent(hwnd) {
        if let Some(d) = (GetWindowLongPtrW(parent, GWLP_USERDATA) as *const Data).as_ref() {
            let result = match msg {
                WM_SETFOCUS => scroll::reveal(d, hwnd),
                WM_MOUSEWHEEL => scroll::wheel(d, wp),
                _ => Ok(()),
            };
            if let Err(error) = result {
                eprintln!("neuro-mini-properties-scroll: {error}");
            }
            if msg == WM_MOUSEWHEEL {
                return LRESULT(0);
            }
        }
    }
    if msg == WM_NCDESTROY {
        let _ = RemoveWindowSubclass(hwnd, Some(procedure), ID);
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}
