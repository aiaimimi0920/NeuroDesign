//! 有界预览卡片 child；宿主只提供只读预览和点击凭证，不向主题传业务状态。
use crate::{layout::px, Frame, PreviewGridLayout};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use windows::{
    core::{w, Error, Result, HSTRING},
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Controls::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};
#[path = "preview_grid_paint.rs"]
mod paint;
#[cfg(test)]
#[path = "preview_grid_tests.rs"]
mod tests;
#[path = "preview_grid_window.rs"]
mod window;
const CLASS: windows::core::PCWSTR = w!("NeuroMini.Native.PreviewGrid");
/// 本 UI 线程上的已准备预览；不得在绘制中读取文件、联网或派发窗口消息。
pub trait CardPreview {
    /// # Safety
    /// dc/rect 仅在本次同步 paint 期间有效，不保存 HDC 或改变它的所有权。
    unsafe fn paint(&self, dc: HDC, rect: RECT);
}
#[derive(Clone)]
pub struct PreviewCard {
    pub title: String,
    pub detail: String,
    pub enabled: bool,
    pub preview: Option<Rc<dyn CardPreview>>,
}
struct Data {
    parent: HWND,
    notify: u32,
    context: Cell<u32>,
    title: RefCell<String>,
    status: RefCell<String>,
    cards: RefCell<Vec<PreviewCard>>,
    buttons: Vec<HWND>,
    selected: Cell<Option<usize>>,
    layout: RefCell<PreviewGridLayout>,
    drag: Cell<Option<(i32, f64, f64)>>,
    font: Cell<HFONT>,
    lifetime: Rc<Cell<bool>>,
}
impl Drop for Data {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.font.get().0));
        }
    }
}
pub struct PreviewGrid {
    hwnd: HWND,
    identity: isize,
    lifetime: Rc<Cell<bool>>,
}
impl PreviewGrid {
    /// # Safety
    /// parent 属于当前 UI 线程；支持父先销毁。notify 为专属 WM_APP 消息。
    pub unsafe fn new(parent: HWND, notify: u32, cards: Vec<PreviewCard>) -> Result<Self> {
        if !IsWindow(Some(parent)).as_bool()
            || !(WM_APP..0xc000).contains(&notify)
            || cards.is_empty()
            || cards.len() > 64
            || cards.iter().any(|c| {
                [&c.title, &c.detail]
                    .iter()
                    .any(|s| s.encode_utf16().count() > 512 || s.contains('\0'))
            })
        {
            return Err(Error::from(E_INVALIDARG));
        }
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        if RegisterClassW(&WNDCLASSW {
            hInstance: instance,
            lpszClassName: CLASS,
            lpfnWndProc: Some(window::procedure),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        }) == 0
            && GetLastError() != ERROR_CLASS_ALREADY_EXISTS
        {
            return Err(Error::from_win32());
        }
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            CLASS,
            w!("预览"),
            WS_CHILD | WS_CLIPCHILDREN,
            0,
            0,
            1,
            1,
            Some(parent),
            None,
            Some(instance),
            None,
        )?;
        let mut owner = Self {
            hwnd,
            identity: 0,
            lifetime: Rc::new(Cell::new(true)),
        };
        let mut buttons = Vec::new();
        for (index, card) in cards.iter().enumerate() {
            buttons.push(CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                &HSTRING::from(format!("{} {}", card.title, card.detail)),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
                0,
                0,
                1,
                1,
                Some(hwnd),
                Some(HMENU((index + 1) as *mut _)),
                Some(instance),
                None,
            )?);
        }
        let font = CreateFontW(
            -10,
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
            parent,
            notify,
            context: Cell::new(0),
            title: RefCell::default(),
            status: RefCell::default(),
            cards: RefCell::new(cards),
            buttons,
            selected: Cell::new(None),
            layout: RefCell::new(PreviewGridLayout::new(0, 96, 0.0)),
            drag: Cell::new(None),
            font: Cell::new(font),
            lifetime: owner.lifetime.clone(),
        });
        owner.identity = Box::into_raw(data) as isize;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, owner.identity);
        Ok(owner)
    }
    pub fn handle(&self) -> HWND {
        self.hwnd
    }
    /// # Safety
    /// 仅在创建线程调用。父窗口可先销毁；不解引用已复用 HWND。
    pub unsafe fn alive(&self) -> bool {
        self.lifetime.get()
            && self.identity != 0
            && IsWindow(Some(self.hwnd)).as_bool()
            && GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) == self.identity
    }
    unsafe fn data(&self) -> Result<&Data> {
        if self.alive() {
            Ok(&*(self.identity as *const Data))
        } else {
            Err(Error::from(E_HANDLE))
        }
    }
    /// 同一组卡片原位更新，不因选择或 enable 改变几何、重建按钮。
    /// # Safety
    /// 与 new 相同；宿主在内容/资格/显示换代后使用新 context，收到通知仍须复核。
    pub unsafe fn update(
        &self,
        title: &str,
        status: &str,
        enabled: &[bool],
        selected: Option<usize>,
        context: u32,
        dpi: u32,
    ) -> Result<PreviewGridLayout> {
        let data = self.data()?;
        if context == 0
            || enabled.len() != data.buttons.len()
            || selected.is_some_and(|i| i >= enabled.len())
            || [title, status]
                .iter()
                .any(|s| s.encode_utf16().count() > 512 || s.contains('\0'))
        {
            return Err(Error::from(E_INVALIDARG));
        }
        let changed = *data.title.borrow() != title
            || *data.status.borrow() != status
            || data.selected.get() != selected
            || data.context.get() != context;
        *data.title.borrow_mut() = title.into();
        *data.status.borrow_mut() = status.into();
        data.selected.set(selected);
        data.context.set(context);
        for (index, hwnd) in data.buttons.iter().enumerate() {
            data.cards.borrow_mut()[index].enabled = enabled[index];
            if IsWindowEnabled(*hwnd).as_bool() != enabled[index] {
                let _ = EnableWindow(*hwnd, enabled[index]);
            }
            if changed {
                let _ = InvalidateRect(Some(*hwnd), None, false);
            }
        }
        let next = PreviewGridLayout::new(enabled.len(), dpi, data.layout.borrow().offset);
        if data.layout.borrow().dpi != next.dpi {
            let font = CreateFontW(
                -px(10, next.dpi),
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
            let _ = DeleteObject(HGDIOBJ(data.font.replace(font).0));
        }
        if *data.layout.borrow() != next {
            *data.layout.borrow_mut() = next;
            window::place(self.hwnd, data)?;
        }
        if changed {
            let _ = InvalidateRect(Some(self.hwnd), None, false);
        }
        Ok(data.layout.borrow().clone())
    }
    /// # Safety
    /// 与 new 相同；键盘焦点滚入当前局部视口，不移动宿主窗口。
    pub unsafe fn translate(&self, msg: &MSG) -> bool {
        if !self.alive() || !(msg.hwnd == self.hwnd || IsChild(self.hwnd, msg.hwnd).as_bool()) {
            return false;
        }
        let handled = IsDialogMessageW(self.hwnd, msg).as_bool();
        if handled {
            window::reveal(self.hwnd, self.data().unwrap(), GetFocus());
        }
        handled
    }
}
impl Drop for PreviewGrid {
    fn drop(&mut self) {
        unsafe {
            if self.identity == 0 || self.alive() {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}
