//! 附属于工具条的紧凑属性行。只持有控件值和短暂草稿，不认识业务工具或文档。
use crate::{paint::Painter, toolbar, Icon};
use std::cell::{Cell, RefCell};
use windows::{
    core::{w, Error, Result, HSTRING},
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::{
            Controls::*, HiDpi::GetDpiForWindow, Input::KeyboardAndMouse::*, WindowsAndMessaging::*,
        },
    },
};
#[path = "property_attachment.rs"]
mod attachment;
#[path = "property_controls.rs"]
mod controls;
pub use attachment::{attached_position, sync_attached};
#[path = "property_edit.rs"]
mod edit;
#[path = "property_focus.rs"]
mod focus;
#[path = "property_layout.rs"]
mod layout;
#[path = "property_paint.rs"]
mod paint;
#[path = "property_panel.rs"]
mod panel;
#[path = "property_ratio.rs"]
mod ratio;
#[path = "property_scroll.rs"]
mod scroll;
#[path = "property_window.rs"]
mod window;
pub use panel::{PropertyPanel, PANEL, PANEL_CLOSE};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyKind {
    /// 无交互的紧凑状态/行标签；不进入 Tab 顺序，不生成属性提交。
    Label {
        value: String,
        width: u16,
    },
    Number {
        value: i32,
        min: i32,
        max: i32,
        /// 独立绘制的单位；不进入 EDIT 草稿或提交字符串。
        suffix: &'static str,
    },
    ReadOnly {
        value: String,
    },
    Text {
        value: String,
        width: u16,
    },
    /// 宽度和高度独立编辑，约束开关与两组步进保持同一紧凑组。
    Ratio {
        value: String,
        active: bool,
    },
    Action {
        value: String,
        active: bool,
        width: u16,
    },
    Dropdown {
        value: String,
        width: u16,
    },
    Color {
        rgb: Option<u32>,
        stroke: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyField {
    pub id: u16,
    pub label: &'static str,
    pub icon: Option<Icon>,
    pub kind: PropertyKind,
}
#[derive(Clone, Debug)]
pub struct PropertyEdit {
    pub context: u64,
    pub id: u16,
    pub value: Option<String>,
    /// 复合比例组的最终约束状态；与数值一起提交，避免失焦提交和开关点击互相覆盖。
    pub active: Option<bool>,
}
pub const PROPERTY: isize = 4;
struct Control {
    field: PropertyField,
    label: HSTRING,
    hwnd: HWND,
    step: [HWND; 2],
    ratio: Option<ratio::Inputs>,
    frame: RECT,
    dirty: bool,
    pending: Option<PropertyEdit>,
    expanded: bool,
}
impl Control {
    fn handles(&self) -> impl Iterator<Item = HWND> + '_ {
        std::iter::once(self.hwnd)
            .chain(self.step)
            .chain(self.ratio.iter().flat_map(|r| r.handles()))
            .filter(|h| !h.is_invalid())
    }
}
struct Data {
    toolbar: HWND,
    stack_width: Option<u16>,
    context: Cell<u64>,
    tip: RefCell<Option<toolbar::tooltip::Tooltip>>,
    controls: RefCell<Vec<Control>>,
    section_starts: RefCell<Vec<u16>>,
    active: Cell<bool>,
    updating: Cell<bool>,
    painter: Painter,
    join: Cell<crate::strip_shell::Join>,
    font: Cell<HFONT>,
    small_font: Cell<HFONT>,
    brush: HBRUSH,
    dpi: Cell<u32>,
    available: Cell<i32>,
    width: Cell<i32>,
    height: Cell<i32>,
    viewport: Cell<HWND>,
    max_height: Cell<i32>,
    available_height: Cell<i32>,
    content_height: Cell<i32>,
    scroll_top: Cell<f64>,
    scroll_drag: Cell<Option<(i32, f64)>>,
}
pub(crate) struct Bar {
    hwnd: HWND,
    data: Box<Data>,
}
impl Drop for Bar {
    fn drop(&mut self) {
        unsafe {
            if self.alive() {
                SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
                let _ = DestroyWindow(self.hwnd);
            }
            let _ = DeleteObject(HGDIOBJ(self.data.font.get().0));
            let _ = DeleteObject(HGDIOBJ(self.data.small_font.get().0));
            let _ = DeleteObject(HGDIOBJ(self.data.brush.0));
        }
    }
}
impl Bar {
    unsafe fn alive(&self) -> bool {
        GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) == &*self.data as *const Data as isize
    }
    pub(crate) unsafe fn visible(&self, show: bool) {
        if self.alive() {
            let _ = ShowWindow(
                self.hwnd,
                if show && self.data.active.get() {
                    SW_SHOWNOACTIVATE
                } else {
                    SW_HIDE
                },
            );
        }
    }
    pub(crate) unsafe fn place(&self) -> Result<()> {
        attachment::place(self)
    }
}
/// 设置附属属性栏；同 schema 不重建控件，同 context 不覆盖用户尚未提交的草稿。
/// # Safety
/// 与工具条 create 相同；context 必须在业务目标改变时更新。通知仍发到主条的 owner。
pub unsafe fn set_properties(
    hwnd: HWND,
    context: u64,
    fields: Vec<PropertyField>,
    enabled: bool,
) -> Result<()> {
    validate(&fields, 16)?;
    let ui = toolbar::ui(hwnd)
        .ok_or_else(|| Error::new(E_HANDLE, "Floating toolbar no longer exists"))?;
    if fields.is_empty() {
        if let Some(bar) = ui.properties.borrow().as_ref() {
            bar.data.active.set(false);
            bar.visible(false);
        }
        attachment::clear(hwnd)?;
        return Ok(());
    }
    if ui.properties.borrow().is_none() {
        let bar = window::create(hwnd)?;
        *ui.properties.borrow_mut() = Some(bar);
    }
    let properties = ui.properties.borrow();
    let bar = properties.as_ref().unwrap();
    bar.data.active.set(true);
    if bar.data.updating.get() {
        return Ok(());
    }
    let redraw = controls::update(
        bar,
        context,
        fields.into_iter().map(|f| (f, enabled)).collect(),
        Vec::new(),
    )?;
    bar.place()?;
    // 最终根窗口几何也在冻结区内；恢复后由原生合成一次提交整棵 child 树。
    drop(redraw);
    bar.visible(IsWindowVisible(hwnd).as_bool());
    Ok(())
}
fn validate(fields: &[PropertyField], limit: usize) -> Result<()> {
    if fields.len()>limit || fields.iter().enumerate().any(|(i,f)| f.id==0 || f.id>4095 || fields[..i].iter().any(|p|p.id==f.id)
        || matches!(&f.kind,PropertyKind::Label{value,width} if !(16..=512).contains(width) || value.encode_utf16().count()>512 || value.chars().any(char::is_control))
        || matches!(f.kind,PropertyKind::Number{value,min,max,suffix} if min>max || !(min..=max).contains(&value) || suffix.chars().count()>4 || suffix.chars().any(char::is_control))
        || matches!(&f.kind,PropertyKind::Ratio{value,..} if ratio::parts(value).is_none())) {
        return Err(Error::new(E_INVALIDARG,"Invalid compact property fields"));
    }
    Ok(())
}
/// 判断消息目标是否属于工具条的二级属性控件。
/// # Safety
/// 与 set_properties 相同。
pub unsafe fn property_contains(hwnd: HWND, child: HWND) -> bool {
    toolbar::ui(hwnd)
        .and_then(|ui| {
            ui.properties
                .borrow()
                .as_ref()
                .map(|bar| bar.alive() && (child == bar.hwnd || IsChild(bar.hwnd, child).as_bool()))
        })
        .unwrap_or(false)
}
/// 当前附属属性窗口；宿主用于捕获保护和窗口清单，不得改写其 userdata。
/// # Safety
/// 与 set_properties 相同。
pub unsafe fn property_window(hwnd: HWND) -> Option<HWND> {
    let ui = toolbar::ui(hwnd)?;
    let properties = ui.properties.borrow();
    let bar = properties.as_ref()?;
    bar.alive().then_some(bar.hwnd)
}
/// 取出该字段最新一次提交；同字段快速步进合并通知，但最终控件值不丢失。
/// # Safety
/// 只能在创建工具条的线程调用；调用方还须复核 context 和业务目标凭证。
pub unsafe fn take_property_edit(hwnd: HWND, id: u16) -> Option<PropertyEdit> {
    let ui = toolbar::ui(hwnd)?;
    let properties = ui.properties.borrow();
    let bar = properties.as_ref()?;
    let mut controls = bar.data.controls.borrow_mut();
    controls
        .iter_mut()
        .find(|c| c.field.id == id)?
        .pending
        .take()
}
/// 返回当前字段的屏幕矩形，用于将颜色/字体/线型选择附着到触发控件。
/// # Safety
/// 与 set_properties 相同。
pub unsafe fn property_anchor(hwnd: HWND, id: u16) -> Option<RECT> {
    let ui = toolbar::ui(hwnd)?;
    let properties = ui.properties.borrow();
    let bar = properties.as_ref()?;
    let controls = bar.data.controls.borrow();
    let c = controls.iter().find(|c| c.field.id == id)?;
    let mut rect = RECT::default();
    GetWindowRect(c.hwnd, &mut rect).ok()?;
    Some(rect)
}
/// 投影下拉的展开箭头；宿主在短暂模态退出（包括失败/取消）后归还，不改变其它字段外观。
/// # Safety
/// 与 set_properties 相同。
pub unsafe fn set_property_expanded(hwnd: HWND, id: u16, expanded: bool) -> Result<()> {
    let ui = toolbar::ui(hwnd).ok_or_else(|| Error::new(E_HANDLE, "Toolbar no longer exists"))?;
    let properties = ui.properties.borrow();
    if let Some(bar) = properties.as_ref() {
        if let Some(c) = bar
            .data
            .controls
            .borrow_mut()
            .iter_mut()
            .find(|c| c.field.id == id && matches!(c.field.kind, PropertyKind::Dropdown { .. }))
        {
            if c.expanded != expanded {
                c.expanded = expanded;
                let _ = InvalidateRect(Some(c.hwnd), None, false);
            }
        }
    }
    Ok(())
}
/// 属性 EDIT 的 Enter/Escape/Tab 先于宿主快捷键；返回 true 表示消息已消费。
/// # Safety
/// 与 set_properties 相同；message 来自该 UI 线程的真实消息泵。
pub unsafe fn translate_properties(hwnd: HWND, message: &MSG) -> bool {
    let Some(ui) = toolbar::ui(hwnd) else {
        return false;
    };
    let properties = ui.properties.borrow();
    let Some(bar) = properties.as_ref() else {
        return false;
    };
    translate(bar, message)
}
unsafe fn translate(bar: &Bar, message: &MSG) -> bool {
    if !bar.alive()
        || !IsWindowVisible(bar.hwnd).as_bool()
        || !(message.hwnd == bar.hwnd || IsChild(bar.hwnd, message.hwnd).as_bool())
    {
        return false;
    }
    if message.message == WM_KEYDOWN && message.wParam.0 == VK_ESCAPE.0 as usize {
        let input = bar.data.controls.borrow().iter().any(|c| {
            (c.hwnd == message.hwnd
                || c.ratio
                    .as_ref()
                    .is_some_and(|r| r.values.contains(&message.hwnd)))
                && matches!(
                    c.field.kind,
                    PropertyKind::Number { .. }
                        | PropertyKind::Text { .. }
                        | PropertyKind::Ratio { .. }
                )
        });
        if !input {
            SendMessageW(bar.hwnd, WM_CLOSE, None, None);
            return true;
        }
    }
    if matches!(message.message, WM_KEYDOWN | WM_KEYUP | WM_CHAR) {
        // EDIT 子类先收到 Enter/Escape，避免 IsDialogMessage 把它当整个工具条的关闭动作。
        if message.message != WM_KEYDOWN
            || matches!(message.wParam.0 as u16,k if k==VK_RETURN.0 || k==VK_ESCAPE.0 || k==VK_UP.0 || k==VK_DOWN.0)
        {
            let _ = TranslateMessage(message);
            DispatchMessageW(message);
            return true;
        }
    }
    let handled = IsDialogMessageW(bar.hwnd, message).as_bool();
    if handled {
        if let Err(error) = scroll::reveal(&bar.data, GetFocus()) {
            eprintln!("neuro-mini-properties-focus: {error}");
        }
    }
    handled
}
#[cfg(test)]
#[path = "property_ratio_tests.rs"]
mod ratio_tests;
#[cfg(test)]
#[path = "property_switch_tests.rs"]
mod switch_tests;
#[cfg(test)]
#[path = "property_tests.rs"]
mod tests;
