//! owner 绑定的无标题工具条；隐藏宿主不分配绘图器、提示或子控件。
use super::{paint::Painter, Icon, Layout};
use std::cell::{Cell, RefCell};
use windows::{
    core::{w, Error, Result, HSTRING},
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{HiDpi::*, WindowsAndMessaging::*},
    },
};
#[path = "toolbar_content.rs"]
mod content;
#[cfg(test)]
#[path = "toolbar_slots_tests.rs"]
mod slot_tests;
#[path = "toolbar_slots.rs"]
mod slots;
#[path = "tooltip.rs"]
pub(crate) mod tooltip;
#[path = "toolbar_window.rs"]
mod window;
pub use slots::{set_text_slots, TextSlot};
#[derive(Clone, Copy)]
pub struct Button {
    pub id: u16,
    pub label: &'static str,
    pub icon: Icon,
    pub dropdown: bool,
}
#[derive(Clone, Copy)]
pub enum Item {
    Button(Button),
    Divider,
    /// 预留但不占空间的文字工具槽；内容由 set_text_slots 原位投影。
    Slot(u16),
}
struct Control {
    spec: Button,
    hwnd: HWND,
    corner: HWND,
    icon: Cell<Icon>,
    active: Cell<bool>,
    label: RefCell<HSTRING>,
    text: RefCell<Option<HSTRING>>,
    visible: Cell<bool>,
    dropdown: Cell<bool>,
    context: Cell<u32>,
}
struct Content {
    painter: Painter,
    // 系统提示持有 Control 文字的指针，字段析构顺序必须先提示、后文字。
    tip: tooltip::Tooltip,
    controls: Vec<Control>,
    slot_font: RefCell<Option<slots::Font>>,
}
pub(crate) struct Ui {
    pub(crate) owner: HWND,
    pub(crate) notify: u32,
    pub(crate) properties: RefCell<Option<crate::properties::Bar>>,
    pub(crate) join: Cell<crate::strip_shell::Join>,
    content: RefCell<Option<Content>>,
    building: Cell<bool>,
    items: Vec<Item>,
    layout: RefCell<Layout>,
    dpi: Cell<u32>,
    max_width: Cell<i32>,
}
const CLASS: windows::core::PCWSTR = w!("NeuroMini.Native.FloatingToolbar");
pub const CLOSE: isize = 2;
pub const PREPARED: isize = 3;
pub(crate) unsafe fn ui(hwnd: HWND) -> Option<&'static Ui> {
    if !is_toolbar(hwnd) {
        return None;
    }
    (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Ui).as_ref()
}
/// 判断是否为本库的有效工具条。
/// # Safety
/// 只能在创建 HWND 的 UI 线程调用。本库的 GWLP_USERDATA 由本库独占。
pub unsafe fn is_toolbar(hwnd: HWND) -> bool {
    let mut name = [0u16; 64];
    let n = GetClassNameW(hwnd, &mut name) as usize;
    n > 0 && &name[..n] == CLASS.as_wide()
}
/// 返回工具条仍有效的 owner。
/// # Safety
/// 与 `is_toolbar` 相同；不得修改本库窗口的 GWLP_USERDATA。
pub unsafe fn owner(hwnd: HWND) -> Option<HWND> {
    let data = ui(hwnd)?;
    GetWindow(hwnd, GW_OWNER)
        .ok()
        .filter(|owner| *owner == data.owner)
}
/// 创建隐藏工具条。通知 wParam 为按钮 ID；lParam 为 0 主按钮、1 下拉、2 请求关闭、3 完成首次准备。
/// # Safety
/// owner 必须为当前 UI 线程的有效 HWND，notify 必须是宿主拥有的 WM_APP 消息。
/// 调用方拥有返回 HWND，必须在该 UI 线程销毁它；不得在同步通知中销毁工具条。
pub unsafe fn create(owner: HWND, title: &str, notify: u32, items: &[Item]) -> Result<HWND> {
    let buttons: Vec<_> = items.iter().filter_map(|item| slots::spec(*item)).collect();
    if !IsWindow(Some(owner)).as_bool()
        || !(WM_APP..0xc000).contains(&notify)
        || items.len() > 96
        || buttons.is_empty()
        || buttons.len() > 64
        || buttons.iter().enumerate().any(|(i, b)| {
            b.id == 0 || b.id >= 0x8000 || buttons[..i].iter().any(|other| other.id == b.id)
        })
    {
        return Err(Error::new(
            E_INVALIDARG,
            "Invalid floating toolbar controls",
        ));
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
    let dpi = GetDpiForWindow(owner).max(96);
    let layout = Layout::new(items, dpi, i32::MAX);
    let hwnd = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        CLASS,
        &HSTRING::from(title),
        WS_POPUP,
        0,
        0,
        layout.width,
        layout.height,
        Some(owner),
        None,
        Some(instance),
        None,
    )?;
    let data = Box::new(Ui {
        owner,
        notify,
        properties: RefCell::new(None),
        join: Cell::new(crate::strip_shell::Join::default()),
        content: RefCell::new(None),
        building: Cell::new(false),
        items: items.to_vec(),
        layout: RefCell::new(layout),
        dpi: Cell::new(dpi),
        max_width: Cell::new(i32::MAX),
    });
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as isize);
    Ok(hwnd)
}
/// 首次展示前分配真实 BUTTON、tooltip 和绘图器；失败回滚，可再次尝试。
/// # Safety
/// 与 `create` 相同；hwnd 必须是本库未销毁的工具条。
pub unsafe fn prepare(hwnd: HWND) -> Result<()> {
    content::prepare(hwnd)
}
/// 查询是否已完整准备；不触发分配。
/// # Safety
/// 与 `is_toolbar` 相同。
pub unsafe fn ready(hwnd: HWND) -> bool {
    ui(hwnd).is_some_and(|ui| ui.content.borrow().is_some())
}
/// 设置 DPI 和可用宽度；几何未变化时不触发移动、重绘或重建控件。
/// # Safety
/// 与 `prepare` 相同。
pub unsafe fn resize(hwnd: HWND, max_width: i32, dpi: u32) -> Result<()> {
    let data = ui(hwnd).ok_or_else(|| Error::new(E_HANDLE, "Floating toolbar no longer exists"))?;
    data.max_width.set(max_width);
    let next = Layout::new(&slots::items(data), dpi, max_width);
    if *data.layout.borrow() == next && data.dpi.get() == dpi {
        return Ok(());
    }
    let (width, height) = (next.width, next.height);
    SetWindowPos(
        hwnd,
        None,
        0,
        0,
        width,
        height,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
    )?;
    content::arrange(hwnd, &next, dpi)?;
    // 只在窗口和控件布局成功后提交缓存，失败后同参数调用仍会重试。
    *data.layout.borrow_mut() = next;
    data.dpi.set(dpi);
    let _ = InvalidateRect(Some(hwnd), None, false);
    Ok(())
}
/// 更新真实状态与可访问名称；不改变控件几何或焦点。
/// # Safety
/// 与 `prepare` 相同；必须先成功 prepare。
pub unsafe fn update(
    hwnd: HWND,
    id: u16,
    icon: Icon,
    active: bool,
    enabled: bool,
    dropdown_enabled: bool,
    label: &str,
) -> Result<()> {
    let data = ui(hwnd).ok_or_else(|| Error::new(E_HANDLE, "Floating toolbar no longer exists"))?;
    let content = data.content.borrow();
    let content = content
        .as_ref()
        .ok_or_else(|| Error::new(E_UNEXPECTED, "Floating toolbar is not prepared"))?;
    let control = content
        .controls
        .iter()
        .find(|c| c.spec.id == id)
        .ok_or_else(|| Error::new(E_INVALIDARG, "Unknown floating toolbar button"))?;
    let changed = (control.active.replace(active) != active) | (control.icon.replace(icon) != icon);
    for child in [control.hwnd, control.corner]
        .into_iter()
        .filter(|h| !h.is_invalid())
    {
        let enabled = if child == control.corner {
            dropdown_enabled
        } else {
            enabled
        };
        if windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled(child).as_bool() != enabled
        {
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(child, enabled);
        }
        if changed {
            let _ = InvalidateRect(Some(child), None, false);
        }
    }
    if control.label.borrow().to_string_lossy() != label {
        let next = HSTRING::from(label);
        SetWindowTextW(control.hwnd, &next)?;
        content.tip.update(hwnd, control.hwnd, &next);
        *control.label.borrow_mut() = next;
        if !control.corner.is_invalid() {
            SetWindowTextW(control.corner, &HSTRING::from(format!("展开{label}列表")))?;
        }
    }
    Ok(())
}
