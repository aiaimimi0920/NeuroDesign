//! 同前台家族的短暂模态租约；捕获外部点击关闭，字体选择不引入方向键改值/首字母搜索。
use super::*;
const TIMER: usize = 0x4e4c;
unsafe fn family(owner: HWND, foreground: HWND) -> bool {
    !foreground.is_invalid()
        && GetAncestor(owner, GA_ROOTOWNER) == GetAncestor(foreground, GA_ROOTOWNER)
}
pub(super) unsafe fn track(
    owner: HWND,
    anchor: RECT,
    items: Vec<String>,
    selected: Option<usize>,
    foreground: impl Fn() -> HWND,
) -> Result<Option<usize>> {
    track_inner(owner, anchor, items, selected, None, foreground)
}
pub(super) unsafe fn track_options(
    owner: HWND,
    anchor: RECT,
    items: Vec<ListOption>,
    foreground: impl Fn() -> HWND,
) -> Result<Option<usize>> {
    let (items, enabled) = items
        .into_iter()
        .map(|item| (item.label, item.enabled))
        .unzip();
    track_inner(owner, anchor, items, None, Some(enabled), foreground)
}
unsafe fn track_inner(
    owner: HWND,
    anchor: RECT,
    items: Vec<String>,
    selected: Option<usize>,
    enabled: Option<Vec<bool>>,
    foreground: impl Fn() -> HWND,
) -> Result<Option<usize>> {
    if !family(owner, foreground()) {
        return Err(Error::new(E_ACCESSDENIED, "List owner is not foreground"));
    }
    if !GetCapture().is_invalid() {
        return Err(Error::new(E_ABORT, "Another pointer capture is active"));
    }
    let mut popup = if enabled.is_some() {
        Popup::new_bounded(owner, anchor, items, selected, true)?
    } else {
        Popup::new(owner, anchor, items, selected)?
    };
    if let Some(enabled) = enabled {
        popup.data.enabled = enabled;
        scroll::project(&popup.data)?;
    }
    let focus = GetFocus();
    let mut owner_rect = RECT::default();
    GetWindowRect(owner, &mut owner_rect)?;
    let result = (|| {
        let _ = ShowWindow(popup.hwnd, SW_SHOWNOACTIVATE);
        SetCapture(popup.hwnd);
        if GetCapture() != popup.hwnd {
            return Err(Error::new(E_ABORT, "List capture interrupted"));
        }
        if SetTimer(Some(popup.hwnd), TIMER, 50, None) == 0 {
            return Err(Error::from_win32());
        }
        while popup.alive() && !popup.data.closed.get() {
            let mut rect = RECT::default();
            if GetCapture() != popup.hwnd
                || !family(owner, foreground())
                || !IsWindowVisible(owner).as_bool()
                || GetWindowRect(owner, &mut rect).is_err()
                || rect != owner_rect
            {
                break;
            }
            let mut msg = MSG::default();
            let status = GetMessageW(&mut msg, None, 0, 0).0;
            if status == -1 {
                return Err(Error::from_win32());
            }
            if status == 0 {
                PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !popup.alive() || GetCapture() != popup.hwnd || !family(owner, foreground()) {
                break;
            }
            let d = &popup.data;
            match msg.message {
                WM_KEYDOWN => match msg.wParam.0 as u16 {
                    k if k == VK_ESCAPE.0 => break,
                    k if k == VK_TAB.0 => scroll::tab(d, GetKeyState(VK_SHIFT.0 as i32) < 0)?,
                    k if k == VK_RETURN.0 || k == VK_SPACE.0 => {
                        if let Some(i) = d.focus.get() {
                            d.choose(i);
                        }
                    }
                    k => scroll::key(d, k)?,
                },
                WM_SYSKEYDOWN | WM_CANCELMODE | WM_NCLBUTTONDOWN | WM_NCRBUTTONDOWN
                | WM_MBUTTONDOWN => break,
                WM_KEYUP | WM_CHAR | WM_SYSKEYUP => (),
                WM_MOUSEWHEEL => scroll::wheel(d, (msg.wParam.0 >> 16) as i16)?,
                WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN => {
                    let mut point = POINT {
                        x: msg.lParam.0 as i16 as i32,
                        y: (msg.lParam.0 >> 16) as i16 as i32,
                    };
                    if !ClientToScreen(msg.hwnd, &mut point).as_bool()
                        || !ScreenToClient(popup.hwnd, &mut point).as_bool()
                    {
                        break;
                    }
                    let bounds = RECT {
                        left: 0,
                        top: 0,
                        right: d.frame.rect.right - d.frame.rect.left,
                        bottom: d.frame.rect.bottom - d.frame.rect.top,
                    };
                    match msg.message {
                        WM_LBUTTONDOWN | WM_RBUTTONDOWN if !geometry::inside(bounds, point) => {
                            break
                        }
                        WM_LBUTTONDOWN => scroll::down(d, point)?,
                        WM_MOUSEMOVE => scroll::motion(d, point)?,
                        WM_LBUTTONUP => {
                            d.drag.set(None);
                            let hit = d.frame.hit(point, d.top.get(), d.items.len());
                            if let Some(i) = d.pressed.take().filter(|i| Some(*i) == hit) {
                                d.choose(i);
                            }
                        }
                        _ => (),
                    }
                }
                WM_TIMER if msg.hwnd == popup.hwnd && msg.wParam.0 == TIMER => (),
                _ => {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
        if let Some(error) = popup.data.error.borrow_mut().take() {
            return Err(error);
        }
        Ok(popup.data.result.get())
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
