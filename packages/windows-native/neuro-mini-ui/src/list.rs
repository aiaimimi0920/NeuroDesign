//! 有界长选项弹层：像素滚动与固定可见 BUTTON 池，不为每个选项创建 HWND。
use crate::paint::Painter;
use std::cell::{Cell, RefCell};
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
#[path = "list_layout.rs"]
mod geometry;
#[path = "list_input.rs"]
mod input;
#[path = "list_paint.rs"]
mod paint;
#[path = "list_scroll.rs"]
mod scroll;
#[path = "list_window.rs"]
mod window;
#[derive(Clone, Copy)]
struct Drag {
    y: i32,
    top: f64,
    maximum: f64,
    travel: i32,
}
struct Data {
    items: Vec<String>,
    enabled: Vec<bool>,
    selected: Option<usize>,
    frame: geometry::Layout,
    root: Cell<HWND>,
    viewport: Cell<HWND>,
    rows: RefCell<Vec<HWND>>,
    top: Cell<f64>,
    hover: Cell<Option<usize>>,
    focus: Cell<Option<usize>>,
    pressed: Cell<Option<usize>>,
    drag: Cell<Option<Drag>>,
    closed: Cell<bool>,
    result: Cell<Option<usize>>,
    error: RefCell<Option<Error>>,
    painter: Painter,
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
            if self.alive() {
                SetWindowLongPtrW(self.data.viewport.get(), GWLP_USERDATA, 0);
                SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}
impl Popup {
    unsafe fn new(
        owner: HWND,
        anchor: RECT,
        items: Vec<String>,
        selected: Option<usize>,
    ) -> Result<Self> {
        Self::new_bounded(owner, anchor, items, selected, false)
    }
    unsafe fn new_bounded(
        owner: HWND,
        anchor: RECT,
        items: Vec<String>,
        selected: Option<usize>,
        extended: bool,
    ) -> Result<Self> {
        if items.is_empty()
            || items.len() > 16_384
            || selected.is_some_and(|i| i >= items.len())
            || items.iter().any(|s| {
                s.is_empty()
                    || if extended {
                        s.encode_utf16().count() > 512
                    } else {
                        s.len() > 256
                    }
                    || s.contains(['\0', '\r', '\n'])
            })
        {
            return Err(Error::new(E_INVALIDARG, "Invalid scrolling list snapshot"));
        }
        let dpi = GetDpiForWindow(owner).max(96);
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
        let frame = geometry::Layout::new(anchor, info.rcWork, dpi, items.len())?;
        let painter = Painter::new()?;
        let font = CreateFontW(
            -crate::layout::px(11, dpi),
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
        let data = Box::new(Data {
            enabled: vec![true; items.len()],
            items,
            selected,
            frame,
            root: Cell::new(HWND::default()),
            viewport: Cell::new(HWND::default()),
            rows: Default::default(),
            top: Cell::new(0.0),
            hover: Cell::new(None),
            focus: Cell::new(None),
            pressed: Cell::new(None),
            drag: Cell::new(None),
            closed: Cell::new(false),
            result: Cell::new(None),
            error: Default::default(),
            painter,
            font,
        });
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        let class = w!("NeuroMini.Native.ScrollingList");
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
        let r = data.frame.rect;
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class,
            w!("选项"),
            WS_POPUP | WS_CLIPCHILDREN,
            r.left,
            r.top,
            r.right - r.left,
            r.bottom - r.top,
            Some(owner),
            None,
            Some(instance),
            None,
        )?;
        let popup = Self { hwnd, data };
        let d = &popup.data;
        d.root.set(hwnd);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, &**d as *const Data as isize);
        let mut affinity = 0;
        GetWindowDisplayAffinity(owner, &mut affinity)?;
        SetWindowDisplayAffinity(hwnd, WINDOW_DISPLAY_AFFINITY(affinity))?;
        let vp = d.frame.viewport;
        let viewport = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class,
            w!("选项列表"),
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN,
            vp.left,
            vp.top,
            vp.right - vp.left,
            vp.bottom - vp.top,
            Some(hwnd),
            None,
            Some(instance),
            None,
        )?;
        d.viewport.set(viewport);
        SetWindowLongPtrW(viewport, GWLP_USERDATA, &**d as *const Data as isize);
        let count = d
            .items
            .len()
            .min(((vp.bottom - vp.top) / d.frame.row + 2) as usize);
        for slot in 0..count {
            let row = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!(""),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
                0,
                0,
                vp.right - vp.left,
                d.frame.row,
                Some(viewport),
                Some(HMENU((slot + 1) as *mut _)),
                Some(instance),
                None,
            )?;
            d.rows.borrow_mut().push(row);
        }
        let radius = crate::layout::px(8, dpi);
        let region = CreateRoundRectRgn(
            0,
            0,
            r.right - r.left + 1,
            r.bottom - r.top + 1,
            radius,
            radius,
        );
        if region.is_invalid() {
            return Err(Error::from_win32());
        }
        if SetWindowRgn(hwnd, Some(region), false) == 0 {
            let _ = DeleteObject(HGDIOBJ(region.0));
            return Err(Error::from_win32());
        }
        scroll::project(d)?;
        Ok(popup)
    }
    unsafe fn alive(&self) -> bool {
        GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) == &*self.data as *const Data as isize
    }
}
impl Data {
    fn choose(&self, index: usize) {
        if self.enabled.get(index) == Some(&true) {
            self.result.set(Some(index));
            self.closed.set(true);
        }
    }
}
/// 显示 196 DIP 宽、至多 220 DIP 内容高、28 DIP 行的可滚动选项快照。
/// 返回索引或取消；不枚举应用数据，不创建与选项数量等量的窗口。
/// # Safety
/// owner/anchor 属于本 UI 线程和前台 owner 家族；调用方先建业务模态门禁，不能跨调用借用模型。
/// 返回后复核目标凭证。关闭释放捕获、timer、子窗口和字体；继承 owner 的截图排除。
pub unsafe fn track_list(
    owner: HWND,
    anchor: RECT,
    items: Vec<String>,
    selected: Option<usize>,
) -> Result<Option<usize>> {
    input::track(owner, anchor, items, selected, || GetForegroundWindow())
}
pub struct ListOption {
    pub label: String,
    pub enabled: bool,
}
/// 与 track_list 相同的有界滚动列表，另支持真实禁用项；禁用项保留名称，不可鼠标或键盘提交。
/// # Safety
/// 与 track_list 相同。
pub unsafe fn track_options(
    owner: HWND,
    anchor: RECT,
    items: Vec<ListOption>,
) -> Result<Option<usize>> {
    input::track_options(owner, anchor, items, || GetForegroundWindow())
}
#[cfg(test)]
#[path = "list_paint_tests.rs"]
mod paint_tests;
#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
