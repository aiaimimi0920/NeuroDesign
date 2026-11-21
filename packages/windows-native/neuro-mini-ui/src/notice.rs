//! 通知卡的短暂生命周期；稳定子 HWND 由通知栈裁剪，五秒或显式关闭同步失效。
use std::{
    cell::Cell,
    rc::{Rc, Weak},
};
use windows::{
    core::{w, Error, Result, HSTRING},
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{HiDpi::GetDpiForWindow, WindowsAndMessaging::*},
    },
};
#[path = "notice_paint.rs"]
mod paint;
#[path = "notice_scroll.rs"]
mod scroll;
#[path = "notice_stack.rs"]
mod stack;
pub use stack::NoticeStack;
const CLASS: windows::core::PCWSTR = w!("NeuroMini.Native.Notice");
struct Ui {
    live: Weak<Cell<HWND>>,
    claimed: Rc<Cell<bool>>,
    title: Vec<u16>,
    message: Vec<u16>,
    dismiss: Vec<u16>,
    dpi: Cell<u32>,
    measured: Cell<(i32, u32, i32, i32, i32)>,
    alternate: Cell<(i32, u32, i32, i32, i32)>,
    title_height: Cell<i32>,
    stack: Weak<stack::Data>,
    scroll: scroll::Scroll,
    pressed: Cell<Option<POINT>>,
}
pub struct Notice {
    live: Rc<Cell<HWND>>,
}
impl Notice {
    unsafe fn new(
        stack: &Rc<stack::Data>,
        title: &str,
        message: &str,
        dismiss: &str,
    ) -> Result<Self> {
        let parent = stack.root.get();
        if parent.is_invalid()
            || title.contains('\0')
            || message.contains('\0')
            || dismiss.contains('\0')
            || title.encode_utf16().count() > 128
            || message.encode_utf16().count() > 2048
            || dismiss.encode_utf16().count() > 32
        {
            return Err(Error::new(E_INVALIDARG, "Invalid notice content"));
        }
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        register(CLASS, procedure, instance, IDC_HAND)?;
        let live = Rc::new(Cell::new(HWND::default()));
        let claimed = Rc::new(Cell::new(false));
        let ui = Box::new(Ui {
            live: Rc::downgrade(&live),
            claimed: claimed.clone(),
            title: title.encode_utf16().collect(),
            message: message.encode_utf16().collect(),
            dismiss: dismiss.encode_utf16().collect(),
            dpi: Cell::new(96),
            measured: Cell::default(),
            alternate: Cell::default(),
            title_height: Cell::new(0),
            stack: Rc::downgrade(stack),
            scroll: scroll::Scroll::default(),
            pressed: Cell::new(None),
        });
        let pointer = Box::into_raw(ui);
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CLASS,
            &HSTRING::from(format!("{title}：{message}")),
            WS_CHILD | WS_TABSTOP | WS_CLIPSIBLINGS,
            0,
            0,
            1,
            1,
            Some(parent),
            None,
            Some(instance),
            Some(pointer.cast()),
        );
        // WM_NCCREATE 一旦接管，WM_NCDESTROY 总会释放；建窗之前失败才由调用者释放。
        let hwnd = match hwnd {
            Ok(hwnd) => hwnd,
            Err(error) => {
                if !claimed.get() {
                    drop(Box::from_raw(pointer));
                }
                return Err(error);
            }
        };
        let notice = Self { live };
        if SetTimer(Some(hwnd), 1, 5000, None) == 0 {
            return Err(Error::from_win32());
        }
        stack
            .entries
            .borrow_mut()
            .insert(0, Rc::downgrade(&notice.live));
        stack::changed(stack)?;
        Ok(notice)
    }
    pub fn window(&self) -> Option<HWND> {
        let hwnd = self.live.get();
        (!hwnd.is_invalid()).then_some(hwnd)
    }
}
unsafe fn register(
    class: windows::core::PCWSTR,
    procedure: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    instance: HINSTANCE,
    cursor: windows::core::PCWSTR,
) -> Result<()> {
    if RegisterClassW(&WNDCLASSW {
        hInstance: instance,
        lpszClassName: class,
        lpfnWndProc: Some(procedure),
        hCursor: LoadCursorW(None, cursor)?,
        ..Default::default()
    }) == 0
        && GetLastError() != ERROR_CLASS_ALREADY_EXISTS
    {
        return Err(Error::from_win32());
    }
    Ok(())
}
unsafe fn measure(hwnd: HWND, ui: &Ui, width: i32, dpi: u32, limit: i32) -> Result<i32> {
    let cached = ui.measured.get();
    if (cached.0, cached.1, cached.2) != (width, dpi, limit) {
        let alternate = ui.alternate.get();
        let measured = if (alternate.0, alternate.1, alternate.2) == (width, dpi, limit) {
            ui.title_height.set(alternate.4);
            alternate
        } else {
            let height = paint::height(hwnd, ui, width, dpi)?;
            let height = if height > limit {
                paint::height(hwnd, ui, width - crate::layout::px(12, dpi), dpi)?
            } else {
                height
            };
            (width, dpi, limit, height, ui.title_height.get())
        };
        // 栈溢出前后只有两种卡宽；同时缓存，避免每次宿主消息都重建字体/重新排全文。
        ui.alternate.set(cached);
        ui.measured.set(measured);
    }
    ui.title_height.set(ui.measured.get().4);
    Ok(ui.measured.get().3)
}
unsafe fn cancel(hwnd: HWND, ui: &Ui) {
    ui.pressed.set(None);
    ui.scroll.cancel(hwnd);
    if windows::Win32::UI::Input::KeyboardAndMouse::GetCapture() == hwnd {
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
    }
}
impl Drop for Notice {
    fn drop(&mut self) {
        unsafe {
            if let Some(hwnd) = self.window() {
                let _ = DestroyWindow(hwnd);
            }
        }
    }
}
unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let pointer = (*(lp.0 as *const CREATESTRUCTW)).lpCreateParams as *mut Ui;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, pointer as isize);
        (*pointer).claimed.set(true);
        if let Some(live) = (*pointer).live.upgrade() {
            live.set(hwnd);
        }
        return LRESULT(1);
    }
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Ui;
    if pointer.is_null() {
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let ui = &*pointer;
    if ui.scroll.input(hwnd, ui.dpi.get(), msg, wp, lp) {
        if ui.pressed.replace(None).is_some() {
            cancel(hwnd, ui);
        }
        let _ = InvalidateRect(Some(hwnd), None, false);
        return LRESULT(0);
    }
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            paint::draw(hwnd, &*pointer);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            ui.pressed.set(Some(POINT {
                x: lp.0 as i16 as i32,
                y: (lp.0 >> 16) as i16 as i32,
            }));
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetCapture(hwnd);
            LRESULT(0)
        }
        WM_MOUSEMOVE if ui.pressed.get().is_some() => {
            let point = ui.pressed.get().unwrap();
            let limit = crate::layout::px(6, ui.dpi.get());
            if (lp.0 as i16 as i32 - point.x).abs() > limit
                || ((lp.0 >> 16) as i16 as i32 - point.y).abs() > limit
            {
                ui.pressed.set(None);
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            if let Some(stack) = ui.stack.upgrade() {
                return SendMessageW(stack.root.get(), msg, Some(wp), Some(lp));
            }
            LRESULT(0)
        }
        WM_LBUTTONUP if ui.pressed.replace(None).is_some() => {
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_TIMER if wp.0 == 1 => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_KEYDOWN if matches!(wp.0, 13 | 32 | 27) => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_SETFOCUS | WM_KILLFOCUS => {
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_CANCELMODE | WM_CAPTURECHANGED => {
            cancel(hwnd, ui);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            let _ = KillTimer(Some(hwnd), 1);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            let ui = Box::from_raw(pointer);
            if let Some(live) = ui.live.upgrade() {
                live.set(HWND::default());
            }
            cancel(hwnd, &ui);
            if let Some(stack) = ui.stack.upgrade() {
                if let Err(error) = stack::changed(&stack) {
                    stack::fail(stack.root.get(), error);
                }
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
#[cfg(test)]
#[path = "notice_tests.rs"]
mod tests;
