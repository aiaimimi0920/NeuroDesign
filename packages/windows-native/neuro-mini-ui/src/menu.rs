//! 工具组选项的短菜单；只返回选项 ID，应用在模态门禁释放后复核并执行。
use super::{paint::Painter, Icon};
use std::cell::Cell;
use windows::{
    core::{w, Error, Result, HSTRING},
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::*, HiDpi::GetDpiForWindow, Input::KeyboardAndMouse::*, WindowsAndMessaging::*,
        },
    },
};
#[path = "menu_input.rs"]
mod input;
#[path = "menu_paint.rs"]
mod paint;
#[path = "menu_window.rs"]
mod window;
#[derive(Clone)]
pub struct MenuItem {
    pub id: u16,
    pub label: String,
    pub shortcut: String,
    pub icon: Icon,
    pub selected: bool,
    pub enabled: bool,
}
struct Data {
    items: Vec<MenuItem>,
    painter: Painter,
    quiet: bool,
    dpi: u32,
    width: i32,
    height: i32,
    above: bool,
    pointer: i32,
    focus: Cell<Option<usize>>,
    closed: Cell<bool>,
    command: Cell<Option<u16>>,
    font: HFONT,
}
struct Popup {
    hwnd: HWND,
    data: Box<Data>,
}
impl Drop for Data {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.font.0));
        }
    }
}
impl Drop for Popup {
    fn drop(&mut self) {
        unsafe {
            if GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) == &*self.data as *const Data as isize {
                SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}
impl Popup {
    unsafe fn new(owner: HWND, anchor: RECT, items: Vec<MenuItem>, quiet: bool) -> Result<Self> {
        if items.is_empty()
            || items.len() > 12
            || items.iter().enumerate().any(|(i, v)| {
                v.id == 0
                    || items[..i].iter().any(|p| p.id == v.id)
                    || v.label.len() > 256
                    || v.shortcut.len() > 32
            })
        {
            return Err(Error::new(E_INVALIDARG, "Invalid compact menu snapshot"));
        }
        let dpi = GetDpiForWindow(owner).max(96);
        let px = |v| crate::layout::px(v, dpi);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(
            MonitorFromRect(&anchor, MONITOR_DEFAULTTONEAREST),
            &mut info,
        )
        .as_bool()
        {
            return Err(Error::from_win32());
        }
        let work = info.rcWork;
        let font = CreateFontW(
            -px(12),
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            0,
            w!("Segoe UI"),
        );
        if font.is_invalid() {
            return Err(Error::from_win32());
        }
        let dc = GetDC(None);
        let old = SelectObject(dc, HGDIOBJ(font.0));
        let mut width = px(if quiet { 96 } else { 88 });
        for item in &items {
            let text: Vec<_> = item.label.encode_utf16().collect();
            let mut size = SIZE::default();
            if !GetTextExtentPoint32W(dc, &text, &mut size).as_bool() {
                SelectObject(dc, old);
                ReleaseDC(None, dc);
                let _ = DeleteObject(HGDIOBJ(font.0)).ok();
                return Err(Error::from_win32());
            }
            width = width.max(size.cx + px(if item.shortcut.is_empty() { 42 } else { 68 }));
        }
        SelectObject(dc, old);
        ReleaseDC(None, dc);
        let height = px(12 + 20 * items.len() as i32);
        if width > work.right - work.left || height > work.bottom - work.top {
            let _ = DeleteObject(HGDIOBJ(font.0)).ok();
            return Err(Error::new(E_BOUNDS, "Compact menu does not fit work area"));
        }
        let left = anchor.left.clamp(work.left, work.right - width);
        let above = anchor.bottom + px(12) + height > work.bottom;
        let top = if above {
            anchor.top - px(12) - height
        } else {
            anchor.bottom + px(6)
        }
        .clamp(work.top, work.bottom - height);
        let pointer = (anchor.left + px(13) - left).clamp(px(10), width - px(10));
        let data = Box::new(Data {
            items,
            painter: match Painter::new() {
                Ok(p) => p,
                Err(e) => {
                    let _ = DeleteObject(HGDIOBJ(font.0)).ok();
                    return Err(e);
                }
            },
            quiet,
            dpi,
            width,
            height,
            above,
            pointer,
            focus: Cell::new(None),
            closed: Cell::new(false),
            command: Cell::new(None),
            font,
        });
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        let class = w!("NeuroMini.Native.CompactMenu");
        if RegisterClassW(&WNDCLASSW {
            hInstance: instance,
            lpszClassName: class,
            lpfnWndProc: Some(window::procedure),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        }) == 0
            && GetLastError() != ERROR_CLASS_ALREADY_EXISTS
        {
            return Err(Error::from_win32());
        }
        let hwnd = match CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class,
            w!("工具选项"),
            WS_POPUP | WS_CLIPCHILDREN,
            left,
            top,
            width,
            height,
            Some(owner),
            None,
            Some(instance),
            None,
        ) {
            Ok(hwnd) => hwnd,
            Err(e) => {
                return Err(e);
            }
        };
        let popup = Self { hwnd, data };
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, &*popup.data as *const Data as isize);
        for (index, item) in popup.data.items.iter().enumerate() {
            let child = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                &HSTRING::from(&item.label),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
                px(if quiet { 4 } else { 1 }),
                px(if above { 3 } else { 9 }) + index as i32 * px(20),
                width - px(if quiet { 8 } else { 2 }),
                px(20),
                Some(hwnd),
                Some(HMENU((index + 1) as *mut _)),
                Some(instance),
                None,
            )?;
            let _ = EnableWindow(child, item.enabled);
        }
        window::region(&popup)?;
        Ok(popup)
    }
    unsafe fn alive(&self) -> bool {
        GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) == &*self.data as *const Data as isize
    }
}
/// 展示 1–12 项、20 DIP 行高的带指针菜单；不抢其他应用前台、不接管已有鼠标捕获。
/// # Safety
/// owner 与 anchor 属于当前 UI 线程。调用方须先建立应用模态门禁，不能跨调用借用业务模型；
/// 返回后须复核命令目标。菜单只在原前台 owner 家族中存活，所有临时资源在返回前释放。
pub unsafe fn track_menu(
    owner: HWND,
    anchor: RECT,
    items: Vec<MenuItem>,
    quiet: bool,
) -> Result<Option<u16>> {
    input::track(owner, anchor, items, quiet)
}
#[cfg(test)]
#[path = "menu_tests.rs"]
mod tests;
