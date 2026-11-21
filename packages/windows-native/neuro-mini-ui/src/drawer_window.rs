//! 标准按钮只异步通知当前context；窗口回调不调用业务、不持有应用对象。
use super::*;
pub(super) unsafe extern "system" fn procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    std::panic::catch_unwind(|| route(hwnd, msg, wp, lp)).unwrap_or(LRESULT(0))
}
unsafe fn route(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Ui;
    if msg == WM_NCDESTROY {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        if !pointer.is_null() {
            (*pointer).lifetime.set(false);
            drop(Box::from_raw(pointer));
        }
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let Some(data) = pointer.as_ref() else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT | WM_PRINTCLIENT => {
            let mut ps = PAINTSTRUCT::default();
            let dc = if msg == WM_PAINT {
                BeginPaint(hwnd, &mut ps)
            } else {
                HDC(wp.0 as *mut _)
            };
            if let Err(error) = crate::paint_batch::window(hwnd, dc, |out| {
                data.painter.borrow().shell(
                    out,
                    &data.layout.borrow(),
                    data.dpi.get(),
                    data.expanded.get(),
                )
            }) {
                eprintln!("neuro-mini-drawer-paint: {error}");
            }
            if msg == WM_PAINT {
                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        WM_CTLCOLORSTATIC => {
            let dc = HDC(wp.0 as *mut _);
            SetBkMode(dc, TRANSPARENT);
            SetTextColor(dc, COLORREF(0xe1d5cb));
            if let Ok(font) = data.painter.borrow_mut().font(data.dpi.get()) {
                SendMessageW(
                    HWND(lp.0 as *mut _),
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(0)),
                );
            }
            LRESULT(data.painter.borrow().background.0 as isize)
        }
        WM_DRAWITEM if lp.0 != 0 => {
            let item = &*(lp.0 as *const DRAWITEMSTRUCT);
            let spec = data
                .controls
                .iter()
                .find(|c| c.hwnd == item.hwndItem)
                .map(|c| c.spec.borrow().clone());
            if spec.is_some() || item.hwndItem == data.caret {
                if let Err(error) = crate::paint_batch::control(item.hDC, item.rcItem, |out| {
                    let mut item = *item;
                    item.hDC = out;
                    data.painter.borrow().button(
                        &item,
                        spec.as_ref(),
                        data.dpi.get(),
                        data.expanded.get(),
                    )
                }) {
                    eprintln!("neuro-mini-drawer-button: {error}");
                }
            }
            LRESULT(1)
        }
        WM_COMMAND if (wp.0 >> 16) & 0xffff == BN_CLICKED as usize => {
            let id = (wp.0 & 0xffff) as u16;
            let child = HWND(lp.0 as *mut _);
            let valid = if id == DRAWER_TOGGLE {
                child == data.caret
            } else {
                data.expanded.get()
                    && data
                        .controls
                        .iter()
                        .any(|c| c.hwnd == child && c.spec.borrow().id == id)
            };
            if valid && IsWindowEnabled(child).as_bool() && data.context.get() != 0 {
                let _ = PostMessageW(
                    Some(data.parent),
                    data.notify,
                    WPARAM(id as usize | ((data.context.get() as usize) << 32)),
                    LPARAM(hwnd.0 as isize),
                );
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
