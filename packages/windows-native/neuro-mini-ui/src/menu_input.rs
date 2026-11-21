//! 捕获与消息泵的生命周期；外部点击、Esc、失焦、owner 变动均关闭，不重放被消费输入。
use super::*;
const TIMER: usize = 0x4e4d;
unsafe fn family(owner: HWND, foreground: HWND) -> bool {
    !foreground.is_invalid()
        && GetAncestor(owner, GA_ROOTOWNER) == GetAncestor(foreground, GA_ROOTOWNER)
}
pub(super) unsafe fn track(
    owner: HWND,
    anchor: RECT,
    items: Vec<MenuItem>,
    quiet: bool,
) -> Result<Option<u16>> {
    track_with(owner, anchor, items, quiet, || GetForegroundWindow())
}
pub(super) unsafe fn track_with(
    owner: HWND,
    anchor: RECT,
    items: Vec<MenuItem>,
    quiet: bool,
    foreground: impl Fn() -> HWND,
) -> Result<Option<u16>> {
    let current = foreground();
    if !family(owner, current) {
        return Err(Error::new(
            E_ACCESSDENIED,
            "Compact menu owner is not foreground",
        ));
    }
    if !GetCapture().is_invalid() {
        return Err(Error::new(E_ABORT, "Another pointer capture is active"));
    }
    let popup = Popup::new(owner, anchor, items, quiet)?;
    let mut owner_rect = RECT::default();
    GetWindowRect(owner, &mut owner_rect)?;
    let focus = GetFocus();
    let mut pressed = None;
    let result = (|| {
        let _ = ShowWindow(popup.hwnd, SW_SHOWNOACTIVATE);
        SetCapture(popup.hwnd);
        if GetCapture() != popup.hwnd {
            return Err(Error::new(E_ABORT, "Compact menu capture interrupted"));
        }
        if SetTimer(Some(popup.hwnd), TIMER, 50, None) == 0 {
            return Err(Error::from_win32());
        }
        popup.focus(
            popup
                .data
                .items
                .iter()
                .position(|item| item.enabled && item.selected)
                .or_else(|| popup.data.items.iter().position(|item| item.enabled)),
        );
        while !popup.data.closed.get() {
            let mut rect = RECT::default();
            if !popup.alive()
                || GetCapture() != popup.hwnd
                || !family(owner, foreground())
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
            if !popup.alive() || GetCapture() != popup.hwnd || !family(owner, foreground()) {
                break;
            }
            match message.message {
                WM_KEYDOWN => match message.wParam.0 as u16 {
                    k if k == VK_ESCAPE.0 => break,
                    k if k == VK_RETURN.0 || k == VK_SPACE.0 => {
                        popup.choose();
                    }
                    k if k == VK_DOWN.0 || k == VK_TAB.0 => {
                        popup.step(if k == VK_TAB.0 && GetKeyState(VK_SHIFT.0 as i32) < 0 {
                            -1
                        } else {
                            1
                        })
                    }
                    k if k == VK_UP.0 => popup.step(-1),
                    k if k == VK_HOME.0 => {
                        popup.focus(popup.data.items.iter().position(|i| i.enabled))
                    }
                    k if k == VK_END.0 => {
                        popup.focus(popup.data.items.iter().rposition(|i| i.enabled))
                    }
                    _ => {}
                },
                WM_SYSKEYDOWN | WM_CANCELMODE | WM_NCLBUTTONDOWN | WM_NCRBUTTONDOWN
                | WM_MBUTTONDOWN => break,
                WM_CHAR | WM_KEYUP | WM_SYSKEYUP => {}
                WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP => {
                    let mut point = POINT {
                        x: message.lParam.0 as i16 as i32,
                        y: (message.lParam.0 >> 16) as i16 as i32,
                    };
                    if !ClientToScreen(message.hwnd, &mut point).as_bool() {
                        break;
                    }
                    let hit = popup.hit(point);
                    if matches!(message.message, WM_LBUTTONDOWN | WM_RBUTTONDOWN) {
                        if hit.is_none() {
                            break;
                        }
                        pressed = hit;
                    }
                    popup.focus(hit);
                    if matches!(message.message, WM_LBUTTONUP | WM_RBUTTONUP)
                        && pressed.take().is_some_and(|p| Some(p) == hit)
                    {
                        popup.choose();
                    }
                }
                WM_MOUSEWHEEL => {}
                WM_TIMER if message.hwnd == popup.hwnd && message.wParam.0 == TIMER => {}
                _ => {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
        Ok(popup.data.command.get())
    })();
    if popup.alive() {
        let _ = KillTimer(Some(popup.hwnd), TIMER).ok();
        if GetCapture() == popup.hwnd {
            let _ = ReleaseCapture().ok();
        }
    }
    if family(owner, foreground()) && IsWindow(Some(focus)).as_bool() {
        let _ = SetFocus(Some(focus));
    }
    result
}
impl Popup {
    unsafe fn hit(&self, point: POINT) -> Option<usize> {
        self.data
            .items
            .iter()
            .enumerate()
            .find_map(|(index, item)| {
                let child = GetDlgItem(Some(self.hwnd), index as i32 + 1).ok()?;
                let mut rect = RECT::default();
                GetWindowRect(child, &mut rect).ok()?;
                (item.enabled
                    && point.x >= rect.left
                    && point.x < rect.right
                    && point.y >= rect.top
                    && point.y < rect.bottom)
                    .then_some(index)
            })
    }
    unsafe fn focus(&self, index: Option<usize>) {
        if self.data.focus.replace(index) == index {
            return;
        }
        for i in 0..self.data.items.len() {
            if let Ok(child) = GetDlgItem(Some(self.hwnd), i as i32 + 1) {
                let _ = InvalidateRect(Some(child), None, false);
            }
        }
        // NOACTIVATE 菜单使用虚拟行焦点；消息泵处理方向键，不 SetFocus 激活 popup。
    }
    unsafe fn step(&self, direction: isize) {
        let len = self.data.items.len() as isize;
        let start = self
            .data
            .focus
            .get()
            .map_or(if direction > 0 { -1 } else { 0 }, |i| i as isize);
        for step in 1..=len {
            let i = (start + direction * step).rem_euclid(len) as usize;
            if self.data.items[i].enabled {
                self.focus(Some(i));
                break;
            }
        }
    }
    fn choose(&self) {
        if let Some(index) = self.data.focus.get() {
            if self.data.items[index].enabled {
                self.data.command.set(Some(self.data.items[index].id));
                self.data.closed.set(true);
            }
        }
    }
}
