//! 固定槽位的动态文字工具；原位更新、空槽不占宽，主题不持有应用命令或目录。
use super::*;
use windows::Win32::UI::{Controls::DRAWITEMSTRUCT, Input::KeyboardAndMouse::GetFocus};

pub struct TextSlot<'a> {
    pub id: u16,
    pub text: &'a str,
    pub label: &'a str,
    pub dropdown: bool,
    pub active: bool,
    pub enabled: bool,
    /// 异步通知 wParam 高 32 位携带调用方凭证，低 16 位仍为 id。
    pub context: u32,
}
pub(super) fn spec(item: Item) -> Option<Button> {
    match item {
        Item::Button(button) => Some(button),
        Item::Slot(id) => Some(Button {
            id,
            label: "",
            icon: Icon::stroke(""),
            dropdown: true,
        }),
        Item::Divider => None,
    }
}
pub(super) fn items(data: &Ui) -> Vec<Item> {
    let content = data.content.borrow();
    data.items
        .iter()
        .filter_map(|item| match item {
            Item::Slot(id) => content
                .as_ref()?
                .controls
                .iter()
                .find(|c| c.spec.id == *id && c.visible.get())
                .map(|c| Item::Button(c.spec)),
            other => Some(*other),
        })
        .collect()
}
fn valid(text: &str, max: usize) -> bool {
    !text.is_empty() && text.encode_utf16().count() <= max && !text.chars().any(char::is_control)
}
/// 投影全部动态文字槽；未提供的槽隐藏，不重建固定或动态 BUTTON，不丢失其它控件焦点。
/// # Safety
/// 与 update 相同；先成功 prepare。id 必须来自 create 时声明的 Item::Slot，不能在通知中销毁工具条。
pub unsafe fn set_text_slots(hwnd: HWND, slots: &[TextSlot<'_>]) -> Result<()> {
    let data = ui(hwnd).ok_or_else(|| Error::from(E_HANDLE))?;
    if slots.iter().enumerate().any(|(i, slot)| {
        !valid(slot.text, 128)
            || !valid(slot.label, 512)
            || slots[..i].iter().any(|p| p.id == slot.id)
            || !data
                .items
                .iter()
                .any(|item| matches!(item, Item::Slot(id) if *id == slot.id))
    }) {
        return Err(Error::new(E_INVALIDARG, "Invalid text toolbar slots"));
    }
    let mut changed = false;
    {
        let content = data.content.borrow();
        let content = content.as_ref().ok_or_else(|| Error::from(E_UNEXPECTED))?;
        for id in data.items.iter().filter_map(|item| {
            if let Item::Slot(id) = item {
                Some(*id)
            } else {
                None
            }
        }) {
            let control = content.controls.iter().find(|c| c.spec.id == id).unwrap();
            let slot = slots.iter().find(|slot| slot.id == id);
            changed |= control.visible.replace(slot.is_some()) != slot.is_some();
            if let Some(slot) = slot {
                if control.context.get() != slot.context {
                    SetPropW(
                        control.hwnd,
                        crate::button_hold::CONTEXT,
                        Some(HANDLE(slot.context as usize as *mut _)),
                    )?;
                    control.context.set(slot.context);
                    // 内容换代不能把已按下的旧目标续成新命令；只取消本槽手势，不碰其它控件。
                    SendMessageW(control.hwnd, WM_CANCELMODE, None, None);
                    SendMessageW(control.corner, WM_CANCELMODE, None, None);
                }
                crate::button_hold::set_active(control.hwnd, slot.dropdown);
                changed |= control.dropdown.replace(slot.dropdown) != slot.dropdown;
                let text = HSTRING::from(slot.text);
                if control.text.borrow().as_ref() != Some(&text) {
                    *control.text.borrow_mut() = Some(text);
                    let _ = InvalidateRect(Some(control.hwnd), None, false);
                }
                update(
                    hwnd,
                    id,
                    Icon::stroke(""),
                    slot.active,
                    slot.enabled,
                    slot.enabled && slot.dropdown,
                    slot.label,
                )?;
            } else {
                crate::button_hold::set_active(control.hwnd, false);
                if GetFocus() == control.hwnd || GetFocus() == control.corner {
                    let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(data.owner));
                }
            }
        }
    }
    resize(hwnd, data.max_width.get(), data.dpi.get())?;
    if changed {
        content::arrange(hwnd, &data.layout.borrow(), data.dpi.get())?;
    }
    Ok(())
}
pub(super) struct Font {
    handle: HFONT,
    dpi: u32,
}
impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.handle.0));
        }
    }
}
pub(super) unsafe fn paint(
    content: &Content,
    control: &Control,
    item: &DRAWITEMSTRUCT,
    state: crate::paint::ButtonState,
    dpi: u32,
) -> Result<()> {
    if state.corner {
        return Ok(());
    }
    let text = control.text.borrow();
    let Some(text) = text.as_ref() else {
        return Ok(());
    };
    let mut font = content.slot_font.borrow_mut();
    if font.as_ref().is_none_or(|font| font.dpi != dpi) {
        let handle = CreateFontW(
            -crate::layout::px(9, dpi),
            0,
            0,
            0,
            FW_BOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            0,
            w!("Consolas"),
        );
        if handle.is_invalid() {
            return Err(Error::from_win32());
        }
        *font = Some(Font { handle, dpi });
    }
    let old = SelectObject(item.hDC, HGDIOBJ(font.as_ref().unwrap().handle.0));
    SetBkMode(item.hDC, TRANSPARENT);
    let ink = if state.enabled {
        content.painter.palette.signal
    } else {
        content.painter.palette.muted
    };
    SetTextColor(
        item.hDC,
        COLORREF(((ink & 255) << 16) | (ink & 0xff00) | ((ink >> 16) & 255)),
    );
    let mut rect = item.rcItem;
    rect.left += crate::layout::px(3, dpi);
    rect.right -= crate::layout::px(3, dpi);
    DrawTextW(
        item.hDC,
        &mut text.to_vec(),
        &mut rect,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    SelectObject(item.hDC, old);
    Ok(())
}
