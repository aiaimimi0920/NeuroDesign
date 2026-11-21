//! 弹层自己的消息泵；仅在本前台家族中激活，外部输入关闭且不穿透到应用编辑动作。
use super::*;
#[path = "color_pointer.rs"]
pub(super) mod pointer;
const TIMER: usize = 0x4e43;
fn family(owner: HWND, foreground: HWND) -> bool {
    unsafe {
        !foreground.is_invalid()
            && GetAncestor(owner, GA_ROOTOWNER) == GetAncestor(foreground, GA_ROOTOWNER)
    }
}
pub(super) unsafe fn track(
    owner: HWND,
    anchor: RECT,
    options: ColorOptions,
    foreground: impl Fn() -> HWND,
) -> Result<ColorResult> {
    if !family(owner, foreground()) {
        return Err(Error::new(
            E_ACCESSDENIED,
            "Color picker owner is not foreground",
        ));
    }
    if !GetCapture().is_invalid() {
        return Err(Error::new(E_ABORT, "Another pointer capture is active"));
    }
    let popup = Popup::new(owner, anchor, options)?;
    let focus = GetFocus();
    let mut owner_rect = RECT::default();
    GetWindowRect(owner, &mut owner_rect)?;
    let result = (|| {
        let _ = ShowWindow(popup.hwnd, SW_SHOWNORMAL);
        let _ = SetFocus(Some(controls::child(popup.hwnd, controls::HEX)));
        if SetTimer(Some(popup.hwnd), TIMER, 50, None) == 0 {
            return Err(Error::from_win32());
        }
        while popup.alive() && !popup.data.closed.get() {
            let mut rect = RECT::default();
            if !family(owner, foreground())
                || !IsWindowVisible(owner).as_bool()
                || GetWindowRect(owner, &mut rect).is_err()
                || rect != owner_rect
            {
                break;
            }
            let mut message = MSG::default();
            let status = GetMessageW(&mut message, None, 0, 0).0;
            if status == -1 {
                return Err(Error::from_win32());
            }
            if status == 0 {
                PostQuitMessage(message.wParam.0 as i32);
                break;
            }
            if !popup.alive() || !family(owner, foreground()) {
                break;
            }
            let ours = message.hwnd == popup.hwnd || IsChild(popup.hwnd, message.hwnd).as_bool();
            match message.message {
                WM_KEYDOWN if message.wParam.0 == VK_ESCAPE.0 as usize => break,
                WM_KEYDOWN if ours && message.wParam.0 == VK_RETURN.0 as usize => {
                    let focus = GetFocus();
                    if focus == controls::child(popup.hwnd, controls::PALETTE) {
                        palette_key(popup.hwnd, &popup.data, VK_SPACE.0);
                    } else if [
                        controls::APPLY,
                        controls::CANCEL,
                        controls::PREVIEW,
                        controls::PICK,
                    ]
                    .iter()
                    .any(|id| focus == controls::child(popup.hwnd, *id))
                    {
                        SendMessageW(focus, BM_CLICK, None, None);
                    }
                    continue;
                }
                WM_KEYDOWN
                    if ours && GetFocus() == controls::child(popup.hwnd, controls::PALETTE) =>
                {
                    palette_key(popup.hwnd, &popup.data, message.wParam.0 as u16);
                    if message.wParam.0 != VK_TAB.0 as usize {
                        continue;
                    }
                }
                WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_NCLBUTTONDOWN
                | WM_NCRBUTTONDOWN
                    if !ours =>
                {
                    break
                }
                WM_LBUTTONDOWN | WM_RBUTTONDOWN if ours => {
                    let mut point = POINT {
                        x: message.lParam.0 as i16 as i32,
                        y: (message.lParam.0 >> 16) as i16 as i32,
                    };
                    if ClientToScreen(message.hwnd, &mut point).as_bool() {
                        if let Some(point) = pointer::point(popup.hwnd, &popup.data, point) {
                            if pointer::down(
                                popup.hwnd,
                                &popup.data,
                                point,
                                message.message == WM_RBUTTONDOWN,
                            ) {
                                continue;
                            }
                        }
                    }
                }
                WM_MOUSEWHEEL if ours => {
                    let point = POINT {
                        x: message.lParam.0 as i16 as i32,
                        y: (message.lParam.0 >> 16) as i16 as i32,
                    };
                    if pointer::point(popup.hwnd, &popup.data, point)
                        .is_some_and(|p| geometry::inside(geometry::PALETTE, p))
                    {
                        let delta = (message.wParam.0 >> 16) as i16 as i32;
                        popup.data.scroll.set(
                            (popup.data.scroll.get() - delta * 31 / 120)
                                .clamp(0, geometry::max_scroll(popup.data.palette.borrow().len())),
                        );
                        pointer::palette_changed(popup.hwnd, &popup.data);
                    } else {
                        scroll::wheel(
                            popup.hwnd,
                            &popup.data,
                            (message.wParam.0 >> 16) as i16 as i32,
                        );
                    }
                    continue;
                }
                WM_CANCELMODE | WM_SYSKEYDOWN => break,
                _ => (),
            }
            if !IsDialogMessageW(popup.hwnd, &message).as_bool() {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            } else {
                scroll::reveal(popup.hwnd, &popup.data);
            }
        }
        if let Some(error) = popup.data.error.borrow_mut().take() {
            return Err(error);
        }
        Ok(ColorResult {
            action: popup.data.action.get(),
            palette: popup.data.palette.borrow().clone(),
            removed: popup.data.removed.borrow().clone(),
        })
    })();
    if popup.alive() {
        let _ = KillTimer(Some(popup.hwnd), TIMER);
        if GetCapture() == popup.hwnd {
            let _ = ReleaseCapture();
        }
    }
    drop(popup);
    if family(owner, foreground()) && IsWindow(Some(focus)).as_bool() {
        let _ = SetFocus(Some(focus));
    }
    result
}
pub(super) unsafe fn palette_key(hwnd: HWND, d: &Data, key: u16) {
    let len = d.palette.borrow().len();
    let current = d.selected.get().map_or(0, |i| i + 1).min(len);
    match key {
        k if (k == VK_DELETE.0 || k == VK_BACK.0) && current > 0 => pointer::remove(d, current - 1),
        k if k == VK_SPACE.0 && current == 0 => pointer::add(d),
        k if k == VK_SPACE.0 => {
            let mut v = d.value.get();
            let color = d.palette.borrow()[current - 1];
            if color.a == 0 {
                v.a = 0.0;
            } else {
                v.load(color);
            }
            d.value.set(v);
            d.hex_editing.set(false);
        }
        k if [
            VK_LEFT.0, VK_RIGHT.0, VK_UP.0, VK_DOWN.0, VK_HOME.0, VK_END.0,
        ]
        .contains(&k) =>
        {
            let next = match k {
                k if k == VK_LEFT.0 => current.saturating_sub(1),
                k if k == VK_RIGHT.0 => (current + 1).min(len),
                k if k == VK_UP.0 => current.saturating_sub(8),
                k if k == VK_DOWN.0 => (current + 8).min(len),
                k if k == VK_HOME.0 => 0,
                _ => len,
            };
            d.selected.set(next.checked_sub(1));
            let r = geometry::swatch(next, d.scroll.get());
            if r.top < geometry::PALETTE.top {
                d.scroll
                    .set((d.scroll.get() + r.top - geometry::PALETTE.top).max(0));
            }
            if r.bottom > geometry::PALETTE.bottom {
                d.scroll
                    .set(d.scroll.get() + r.bottom - geometry::PALETTE.bottom);
            }
        }
        _ => (),
    }
    controls::refresh(hwnd, d);
    pointer::palette_changed(hwnd, d);
}
