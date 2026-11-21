//! 标准 BUTTON 保留系统键盘与可访问名称；管理 hover 失效并保留 owner-draw 完整帧。
use windows::{
    core::Result,
    Win32::{
        Foundation::*,
        Graphics::Gdi::InvalidateRect,
        UI::{
            Controls::WM_MOUSELEAVE, Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*,
        },
    },
};
const HOVER: windows::core::PCWSTR = windows::core::w!("NeuroMini.Button.Hover");
const ID: usize = 0x4e4d;
/// 保留 owner-draw BUTTON 的完整帧和系统键盘行为。
/// # Safety
/// hwnd 是调用线程拥有的标准 owner-draw BUTTON，父窗口负责完整绘制。
pub unsafe fn install(hwnd: HWND) -> Result<()> {
    SetWindowSubclass(hwnd, Some(procedure), ID, 0).ok()
}
pub(crate) unsafe fn hot(hwnd: HWND) -> bool {
    !GetPropW(hwnd, HOVER).is_invalid()
}
unsafe fn invalidate(hwnd: HWND) {
    let _ = InvalidateRect(Some(hwnd), None, false);
    let id = GetDlgCtrlID(hwnd);
    if let Ok(parent) = GetParent(hwnd) {
        if let Ok(other) = GetDlgItem(Some(parent), id ^ 0x8000) {
            let _ = InvalidateRect(Some(other), None, false);
        }
    }
}
unsafe extern "system" fn procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    match msg {
        // OWNERDRAW 已绘满背景；系统先擦成纯色会在焦点/按下/标签切换间暴露空按钮。
        WM_ERASEBKGND => return LRESULT(1),
        WM_MOUSEMOVE if !hot(hwnd) => {
            let _ = SetPropW(
                hwnd,
                HOVER,
                Some(HANDLE(std::ptr::without_provenance_mut(1))),
            );
            let mut track = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                ..Default::default()
            };
            let _ = TrackMouseEvent(&mut track);
            invalidate(hwnd);
        }
        WM_MOUSELEAVE | WM_KILLFOCUS | WM_NCDESTROY => {
            let _ = RemovePropW(hwnd, HOVER);
            if msg == WM_NCDESTROY {
                let _ = RemoveWindowSubclass(hwnd, Some(procedure), ID).ok();
            } else {
                invalidate(hwnd);
            }
        }
        _ => (),
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}
