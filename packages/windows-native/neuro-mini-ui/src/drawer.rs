//! 通用下附着抽屉的真实 child 控件；宿主负责顶层几何、层序、命令与生命周期。
use crate::{drawer_layout::DrawerLayout, paint::Painter, toolbar::tooltip::Tooltip, Icon};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use windows::{
    core::{w, Error, Result, HSTRING},
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Controls::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};
#[path = "drawer_paint.rs"]
mod paint;
#[cfg(test)]
#[path = "drawer_paint_tests.rs"]
mod paint_tests;
#[cfg(test)]
#[path = "drawer_tests.rs"]
mod tests;
#[path = "drawer_window.rs"]
mod window;
const CLASS: windows::core::PCWSTR = w!("NeuroMini.Native.BottomDrawer");
pub const DRAWER_TOGGLE: u16 = 1;
#[derive(Clone)]
pub struct DrawerButton {
    pub id: u16,
    pub label: String,
    pub icon: Icon,
    pub text: Option<String>,
    pub active: bool,
    pub enabled: bool,
}
struct Control {
    hwnd: HWND,
    spec: RefCell<DrawerButton>,
    label: RefCell<HSTRING>,
}
struct Ui {
    lifetime: Rc<Cell<bool>>,
    parent: HWND,
    notify: u32,
    // tooltip 必须先于其持有指针的文字释放。
    tip: Tooltip,
    controls: Vec<Control>,
    title: HWND,
    name: RefCell<HSTRING>,
    caret: HWND,
    painter: RefCell<paint::Paint>,
    layout: RefCell<DrawerLayout>,
    dpi: Cell<u32>,
    expanded: Cell<bool>,
    context: Cell<u32>,
    title_width: Cell<i32>,
}
pub struct Drawer {
    hwnd: HWND,
    identity: isize,
    lifetime: Rc<Cell<bool>>,
}
impl Drawer {
    /// # Safety
    /// parent 为本 UI 线程的有效 HWND，notify 属于宿主 WM_APP 范围；本对象先于 parent 析构。
    pub unsafe fn new(parent: HWND, notify: u32, buttons: &[DrawerButton]) -> Result<Self> {
        if !IsWindow(Some(parent)).as_bool()
            || !(WM_APP..0xc000).contains(&notify)
            || buttons.is_empty()
            || buttons.len() > 8
            || buttons
                .iter()
                .enumerate()
                .any(|(i, b)| b.id <= DRAWER_TOGGLE || buttons[..i].iter().any(|a| a.id == b.id))
        {
            return Err(Error::new(E_INVALIDARG, "Invalid drawer controls"));
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
            w!("属性"),
            WS_CHILD | WS_CLIPCHILDREN,
            0,
            0,
            20,
            10,
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
        let tip = Tooltip::new(hwnd)?;
        let title = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            None,
            // SS_CENTERIMAGE | SS_ENDELLIPSIS | SS_NOPREFIX；不为三个静态样式扩展Win32依赖面。
            WS_CHILD | WINDOW_STYLE(0x0200 | 0x4000 | 0x0080),
            0,
            0,
            1,
            1,
            Some(hwnd),
            None,
            Some(instance),
            None,
        )?;
        let caret = child(hwnd, DRAWER_TOGGLE, "展开属性")?;
        let mut controls = Vec::new();
        for spec in buttons {
            let control = child(hwnd, spec.id, &spec.label)?;
            let label = HSTRING::from(&spec.label);
            tip.add(hwnd, control, &label)?;
            controls.push(Control {
                hwnd: control,
                spec: RefCell::new(spec.clone()),
                label: RefCell::new(label),
            });
        }
        let data = Box::new(Ui {
            lifetime: owner.lifetime.clone(),
            parent,
            notify,
            tip,
            controls,
            title,
            name: RefCell::new(HSTRING::new()),
            caret,
            painter: RefCell::new(paint::Paint::new()?),
            layout: RefCell::new(DrawerLayout::new(12, &[], false, 96)),
            dpi: Cell::new(96),
            expanded: Cell::new(false),
            context: Cell::new(0),
            title_width: Cell::new(12),
        });
        owner.identity = Box::into_raw(data) as isize;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, owner.identity);
        Ok(owner)
    }
    pub fn handle(&self) -> HWND {
        self.hwnd
    }
    /// # Safety
    /// 仅在创建本控件的UI线程查询，不解引用已退休或已复用HWND的状态。
    pub unsafe fn alive(&self) -> bool {
        self.data().is_ok()
    }
    unsafe fn data(&self) -> Result<&Ui> {
        if !self.lifetime.get()
            || !IsWindow(Some(self.hwnd)).as_bool()
            || GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) != self.identity
            || self.identity == 0
        {
            return Err(Error::from(E_HANDLE));
        }
        Ok(&*(self.identity as *const Ui))
    }
    /// 原位投影，context 隔离旧点击；不执行宿主动作、不重建按钮或抢前台。
    /// # Safety
    /// 与 new 相同；buttons 的稳定 ID、text 标记必须与创建时一致。
    pub unsafe fn update(
        &self,
        title: &str,
        buttons: &[DrawerButton],
        expanded: bool,
        context: u32,
        dpi: u32,
    ) -> Result<DrawerLayout> {
        let data = self.data()?;
        if context == 0
            || buttons.len() != data.controls.len()
            || buttons.iter().zip(&data.controls).any(|(b, c)| {
                let old = c.spec.borrow();
                b.id != old.id || b.text != old.text
            })
        {
            return Err(Error::from(E_INVALIDARG));
        }
        let initializing = data.context.replace(context) == 0;
        let title_changed = data.name.borrow().to_string_lossy() != title;
        let changed = initializing
            || title_changed
            || data.dpi.get() != dpi
            || data.expanded.get() != expanded
            || buttons.iter().zip(&data.controls).any(|(b, c)| {
                let old = c.spec.borrow();
                old.active != b.active
                    || old.enabled != b.enabled
                    || old.icon != b.icon
                    || old.label != b.label
            });
        if !changed {
            return Ok(data.layout.borrow().clone());
        }
        let first = GetWindowLongW(self.hwnd, GWL_STYLE) as u32 & WS_VISIBLE.0 == 0;
        let redraw = crate::paint_batch::Hold::new(self.hwnd);
        if title_changed {
            let name = HSTRING::from(title);
            SetWindowTextW(data.title, &name)?;
            if data.name.borrow().is_empty() {
                data.tip.add(self.hwnd, data.title, &name)?;
            } else {
                data.tip.update(self.hwnd, data.title, &name);
            }
            *data.name.borrow_mut() = name;
        }
        for (spec, control) in buttons.iter().zip(&data.controls) {
            let changed = {
                let old = control.spec.borrow();
                old.active != spec.active || old.enabled != spec.enabled || old.icon != spec.icon
            };
            if control.label.borrow().to_string_lossy() != spec.label {
                let name = HSTRING::from(&spec.label);
                SetWindowTextW(control.hwnd, &name)?;
                data.tip.update(self.hwnd, control.hwnd, &name);
                *control.label.borrow_mut() = name;
            }
            *control.spec.borrow_mut() = spec.clone();
            if IsWindowEnabled(control.hwnd).as_bool() != spec.enabled {
                let _ = EnableWindow(control.hwnd, spec.enabled);
            }
            if changed {
                let _ = InvalidateRect(Some(control.hwnd), None, false);
            }
        }
        if title_changed || data.dpi.get() != dpi {
            data.title_width.set(
                data.painter
                    .borrow_mut()
                    .title_width(self.hwnd, title, dpi)?,
            );
        }
        let width = data.title_width.get();
        let next = DrawerLayout::new(
            width,
            &buttons.iter().map(|b| b.text.is_some()).collect::<Vec<_>>(),
            expanded,
            dpi,
        );
        if *data.layout.borrow() != next || data.dpi.get() != dpi || data.expanded.get() != expanded
        {
            if !expanded && data.controls.iter().any(|c| GetFocus() == c.hwnd) {
                let _ = SetFocus(Some(data.caret));
            }
            SetWindowTextW(
                data.caret,
                if expanded {
                    w!("收起属性")
                } else {
                    w!("展开属性")
                },
            )?;
            data.dpi.set(dpi);
            data.expanded.set(expanded);
            *data.layout.borrow_mut() = next.clone();
            SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                next.width,
                next.height,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREDRAW,
            )?;
            place(data.title, next.title, expanded)?;
            for (control, frame) in data.controls.iter().zip(&next.buttons) {
                place(control.hwnd, *frame, expanded)?;
            }
            place(data.caret, next.caret, true)?;
            next.apply_region(self.hwnd, expanded, dpi)?;
            let _ = InvalidateRect(Some(self.hwnd), None, false);
        }
        drop(redraw);
        if first {
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            crate::paint_batch::present(self.hwnd)?;
        }
        Ok(next)
    }
    /// # Safety
    /// message 为同一 UI 线程消息；只翻译此抽屉自己的标准键盘导航。
    pub unsafe fn translate(&self, message: &MSG) -> bool {
        if self.data().is_err()
            || !IsWindowVisible(self.hwnd).as_bool()
            || !(message.hwnd == self.hwnd || IsChild(self.hwnd, message.hwnd).as_bool())
        {
            return false;
        }
        if message.message == WM_KEYDOWN
            && message.wParam.0 == VK_RETURN.0 as usize
            && message.lParam.0 & (1 << 30) == 0
        {
            SendMessageW(message.hwnd, BM_CLICK, None, None);
            return true;
        }
        IsDialogMessageW(self.hwnd, message).as_bool()
    }
}
unsafe fn child(parent: HWND, id: u16, label: &str) -> Result<HWND> {
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("BUTTON"),
        &HSTRING::from(label),
        WS_CHILD | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
        0,
        0,
        1,
        1,
        Some(parent),
        Some(HMENU(id as *mut _)),
        None,
        None,
    )?;
    crate::button::install(hwnd)?;
    Ok(hwnd)
}
unsafe fn place(hwnd: HWND, f: crate::Frame, visible: bool) -> Result<()> {
    SetWindowPos(
        hwnd,
        None,
        f.x,
        f.y,
        f.width,
        f.height,
        SWP_NOZORDER
            | SWP_NOACTIVATE
            | SWP_NOREDRAW
            | if visible {
                SWP_SHOWWINDOW
            } else {
                SWP_HIDEWINDOW
            },
    )
}
impl Drop for Drawer {
    fn drop(&mut self) {
        unsafe {
            if self.lifetime.get()
                && IsWindow(Some(self.hwnd)).as_bool()
                && GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) == self.identity
            {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}
