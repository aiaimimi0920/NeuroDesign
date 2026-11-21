//! 分体按钮主体的长按租约：450ms、6 DIP 移动容忍；先交还 BUTTON 捕获再异步展开。
use std::cell::Cell;
use windows::{
    core::Result,
    Win32::{
        Foundation::*,
        UI::{Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
    },
};
const ID: usize = 0x4e4f;
const TIMER: usize = 0x4e50;
const SET_ACTIVE: u32 = WM_APP + 0x4e;
pub(crate) const CONTEXT: windows::core::PCWSTR = windows::core::w!("NeuroMini.Button.Context");
struct Hold {
    start: Cell<Option<POINT>>,
    consumed: Cell<bool>,
    tolerance: Cell<i32>,
    enabled: Cell<bool>,
    context: Cell<usize>,
}
pub(crate) unsafe fn install(hwnd: HWND, dpi: u32) -> Result<()> {
    let data = Box::new(Hold {
        start: Cell::new(None),
        consumed: Cell::new(false),
        tolerance: Cell::new(crate::layout::px(6, dpi)),
        enabled: Cell::new(true),
        context: Cell::new(0),
    });
    let pointer = Box::into_raw(data);
    if SetWindowSubclass(hwnd, Some(procedure), ID, pointer as usize).as_bool() {
        Ok(())
    } else {
        drop(Box::from_raw(pointer));
        Err(windows::core::Error::from_win32())
    }
}
pub(crate) unsafe fn set_active(hwnd: HWND, enabled: bool) {
    // 私有 BUTTON 消息直接交给现有 subclass owner，不依赖系统未必按名称导出的 GetWindowSubclass。
    SendMessageW(hwnd, SET_ACTIVE, Some(WPARAM(enabled as usize)), None);
}
unsafe extern "system" fn procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    pointer: usize,
) -> LRESULT {
    let data = &*(pointer as *const Hold);
    if msg == SET_ACTIVE {
        if data.enabled.replace(wp.0 != 0) != (wp.0 != 0) {
            data.start.set(None);
            data.consumed.set(false);
            let _ = KillTimer(Some(hwnd), TIMER);
        }
        return LRESULT(0);
    }
    if !data.enabled.get() && msg != WM_NCDESTROY {
        return DefSubclassProc(hwnd, msg, wp, lp);
    }
    match msg {
        WM_LBUTTONDOWN => {
            data.context.set(GetPropW(hwnd, CONTEXT).0 as usize);
            data.tolerance.set(crate::layout::px(
                6,
                windows::Win32::UI::HiDpi::GetDpiForWindow(hwnd).max(96),
            ));
            data.consumed.set(false);
            data.start.set(Some(POINT {
                x: lp.0 as i16 as i32,
                y: (lp.0 >> 16) as i16 as i32,
            }));
            // 默认 BUTTON 可能同步通知宿主，返回后不再借用 subclass 数据。
            if SetTimer(Some(hwnd), TIMER, 450, None) == 0 {
                data.start.set(None);
            }
            return DefSubclassProc(hwnd, msg, wp, lp);
        }
        WM_MOUSEMOVE
            if data.start.get().is_some_and(|p| {
                (p.x - lp.0 as i16 as i32).abs() > data.tolerance.get()
                    || (p.y - (lp.0 >> 16) as i16 as i32).abs() > data.tolerance.get()
            }) =>
        {
            data.start.set(None);
            let _ = KillTimer(Some(hwnd), TIMER);
        }
        WM_TIMER if wp.0 == TIMER => {
            let _ = KillTimer(Some(hwnd), TIMER);
            if data.start.take().is_some()
                && GetCapture() == hwnd
                && IsWindowEnabled(hwnd).as_bool()
            {
                data.consumed.set(true);
                SendMessageW(hwnd, BM_SETSTATE, Some(WPARAM(0)), None);
                let _ = ReleaseCapture();
                if let Ok(parent) = GetParent(hwnd) {
                    let _ = PostMessageW(
                        Some(parent),
                        WM_COMMAND,
                        WPARAM(
                            (GetDlgCtrlID(hwnd) as u16 | 0x8000) as usize
                                | (data.context.get() << 32),
                        ),
                        LPARAM(hwnd.0 as isize),
                    );
                }
            }
            return LRESULT(0);
        }
        WM_LBUTTONUP => {
            data.start.set(None);
            let _ = KillTimer(Some(hwnd), TIMER);
            if data.consumed.replace(false) {
                return LRESULT(0);
            }
        }
        WM_CANCELMODE | WM_CAPTURECHANGED | WM_KILLFOCUS | WM_ENABLE => {
            data.start.set(None);
            let _ = KillTimer(Some(hwnd), TIMER);
        }
        WM_NCDESTROY => {
            let _ = RemovePropW(hwnd, CONTEXT);
            let _ = KillTimer(Some(hwnd), TIMER);
            let _ = RemoveWindowSubclass(hwnd, Some(procedure), ID);
            drop(Box::from_raw(pointer as *mut Hold));
        }
        _ => {}
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}
