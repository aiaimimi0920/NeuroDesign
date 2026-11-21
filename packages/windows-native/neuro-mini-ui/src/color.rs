//! 独立的原生颜色弹层：只返回确认色和调色板操作，不执行应用样式、采样或持久化。
use crate::{paint::Painter, Icon};
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
#[path = "color_controls.rs"]
mod controls;
#[path = "color_layout.rs"]
mod geometry;
#[path = "color_input.rs"]
mod input;
#[path = "color_paint.rs"]
mod paint;
#[path = "color_scroll.rs"]
mod scroll;
#[path = "color_value.rs"]
mod value;
#[path = "color_window.rs"]
mod window;
pub use value::Rgba;
#[derive(Clone, Copy)]
pub struct ColorIcons {
    pub eyedropper: Icon,
    pub remove: Icon,
    pub apply: Icon,
}
pub struct ColorOptions {
    pub initial: Rgba,
    pub palette: Vec<Rgba>,
    pub screen_pick: bool,
    pub icons: ColorIcons,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorAction {
    Cancel,
    Apply(Rgba),
    ScreenPick,
}
#[derive(Debug)]
pub struct ColorResult {
    pub action: ColorAction,
    pub palette: Vec<Rgba>,
    pub removed: Vec<Rgba>,
}
struct Data {
    value: Cell<value::Hsva>,
    palette: RefCell<Vec<Rgba>>,
    removed: RefCell<Vec<Rgba>>,
    selected: Cell<Option<usize>>,
    scroll: Cell<i32>,
    origin: Cell<POINT>,
    drag: Cell<Option<u8>>,
    updating: Cell<bool>,
    hex_editing: Cell<bool>,
    closed: Cell<bool>,
    action: Cell<ColorAction>,
    error: RefCell<Option<Error>>,
    dpi: u32,
    width: i32,
    height: i32,
    icons: ColorIcons,
    screen_pick: bool,
    painter: Painter,
    font: HFONT,
    brush: HBRUSH,
}
struct Popup {
    hwnd: HWND,
    data: Box<Data>,
}
impl Drop for Data {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.font.0));
            let _ = DeleteObject(HGDIOBJ(self.brush.0));
        }
    }
}
impl Drop for Popup {
    fn drop(&mut self) {
        unsafe {
            if self.alive() {
                SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}
impl Popup {
    unsafe fn new(owner: HWND, anchor: RECT, options: ColorOptions) -> Result<Self> {
        if options.palette.len() > 4096 {
            return Err(Error::new(E_BOUNDS, "Color palette exceeds 4096 entries"));
        }
        for icon in [
            options.icons.eyedropper,
            options.icons.remove,
            options.icons.apply,
        ] {
            icon.validate()?;
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
        let frame = geometry::anchored(anchor, info.rcWork, dpi);
        let painter = Painter::new()?;
        let font = CreateFontW(
            -crate::layout::px(12, dpi),
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
        let brush = CreateSolidBrush(COLORREF(0x201711));
        if brush.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(font.0));
            return Err(Error::from_win32());
        }
        let data = Box::new(Data {
            value: Cell::new(value::Hsva::new(options.initial)),
            palette: RefCell::new(options.palette),
            removed: Default::default(),
            selected: Cell::new(None),
            scroll: Cell::new(0),
            origin: Cell::new(POINT::default()),
            drag: Cell::new(None),
            updating: Cell::new(false),
            hex_editing: Cell::new(false),
            closed: Cell::new(false),
            action: Cell::new(ColorAction::Cancel),
            error: Default::default(),
            dpi,
            width: frame.right - frame.left,
            height: frame.bottom - frame.top,
            icons: options.icons,
            screen_pick: options.screen_pick,
            painter,
            font,
            brush,
        });
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        let class = w!("NeuroMini.Native.ColorPicker");
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
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class,
            w!("颜色选择器"),
            WS_POPUP | WS_CLIPCHILDREN,
            frame.left,
            frame.top,
            data.width,
            data.height,
            Some(owner),
            None,
            Some(instance),
            None,
        )?;
        let popup = Self { hwnd, data };
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, &*popup.data as *const Data as isize);
        let mut affinity = 0;
        GetWindowDisplayAffinity(owner, &mut affinity)?;
        SetWindowDisplayAffinity(hwnd, WINDOW_DISPLAY_AFFINITY(affinity))?;
        let radius = crate::layout::px(18, dpi);
        let region = CreateRoundRectRgn(
            0,
            0,
            popup.data.width + 1,
            popup.data.height + 1,
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
        controls::create(&popup)?;
        scroll::configure(hwnd, &popup.data)?;
        Ok(popup)
    }
    unsafe fn alive(&self) -> bool {
        GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) == &*self.data as *const Data as isize
    }
}
/// 显示附着的 HSV/HEX/alpha 调色板；取消不提交颜色，但增删调色板的动作独立保留。
/// # Safety
/// owner、anchor 属于当前 UI 线程且 owner 位于当前前台家族；调用方先建业务模态门禁，不能跨调用借用模型。
/// 返回后须复核目标凭证；屏幕取色由调用方在门禁释放后启动，不由主题库读取桌面。
pub unsafe fn track_color(owner: HWND, anchor: RECT, options: ColorOptions) -> Result<ColorResult> {
    input::track(owner, anchor, options, || GetForegroundWindow())
}
#[cfg(test)]
#[path = "color_tests.rs"]
mod tests;
