//! 窗口过程只绘制与通知；关闭请求由宿主审核，不绕过应用模态和系统会话门禁。
use super::*;
use windows::Win32::UI::{Controls::*, Input::KeyboardAndMouse::*};
pub(super) unsafe extern "system" fn procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    std::panic::catch_unwind(|| route(hwnd, msg, wp, lp)).unwrap_or(LRESULT(0))
}
unsafe fn route(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCDESTROY {
        let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Ui;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        if !pointer.is_null() {
            drop(Box::from_raw(pointer));
        }
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let Some(data) = ui(hwnd) else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_SHOWWINDOW if wp.0 == 0 => {
            if let Some(bar) = data.properties.borrow().as_ref() {
                bar.visible(false);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_WINDOWPOSCHANGED => {
            if let Some(bar) = data.properties.borrow().as_ref().filter(|_| {
                lp.0 == 0 || !(*(lp.0 as *const WINDOWPOS)).flags.contains(SWP_NOREDRAW)
            }) {
                if let Err(e) = bar.place() {
                    eprintln!("neuro-mini-properties-follow: {e}");
                }
                // PREPARED 同步刷新发生时 WS_VISIBLE 尚未落地；只按最终窗口状态显示附属行。
                bar.visible(IsWindowVisible(hwnd).as_bool());
            }
            let result = DefWindowProcW(hwnd, msg, wp, lp);
            if lp.0 != 0 && (*(lp.0 as *const WINDOWPOS)).flags.contains(SWP_SHOWWINDOW) {
                if let Err(error) = crate::paint_batch::present(hwnd) {
                    eprintln!("neuro-mini-toolbar-show-paint: {error}");
                }
            }
            result
        }
        WM_WINDOWPOSCHANGING if lp.0 != 0 => {
            let pos = &mut *(lp.0 as *mut WINDOWPOS);
            if pos.flags.contains(SWP_SHOWWINDOW) && !pos.flags.contains(SWP_HIDEWINDOW) {
                if let Err(error) = prepare(hwnd) {
                    pos.flags = (pos.flags & !SWP_SHOWWINDOW) | SWP_HIDEWINDOW;
                    eprintln!("neuro-mini-toolbar-prepare: {error}");
                } else {
                    SendMessageW(
                        data.owner,
                        data.notify,
                        Some(WPARAM(0)),
                        Some(LPARAM(PREPARED)),
                    );
                }
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_CTLCOLORBTN => {
            let palette = data
                .content
                .borrow()
                .as_ref()
                .map(|c| c.painter.palette)
                .unwrap_or_default();
            crate::paint_batch::background(HDC(wp.0 as *mut _), palette.surface)
        }
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut paint);
            if let Some(content) = data.content.borrow().as_ref() {
                let layout = data.layout.borrow();
                if let Err(error) = crate::paint_batch::window(hwnd, dc, |out| {
                    content.painter.shell(
                        out,
                        layout.width,
                        layout.height,
                        data.dpi.get(),
                        &layout.dividers,
                        data.join.get(),
                    )
                }) {
                    eprintln!("neuro-mini-toolbar-paint: {error}");
                }
            }
            let _ = EndPaint(hwnd, &paint);
            LRESULT(0)
        }
        WM_DRAWITEM if lp.0 != 0 => {
            let item = &*(lp.0 as *const DRAWITEMSTRUCT);
            if let Some(content) = data.content.borrow().as_ref() {
                if let Some(control) = content
                    .controls
                    .iter()
                    .find(|c| c.hwnd == item.hwndItem || c.corner == item.hwndItem)
                {
                    let state = crate::paint::ButtonState {
                        active: control.active.get(),
                        enabled: IsWindowEnabled(item.hwndItem).as_bool(),
                        focus: item.itemState.0 & ODS_FOCUS.0 != 0,
                        hot: crate::button::hot(control.hwnd) || crate::button::hot(control.corner),
                        pressed: item.itemState.0 & ODS_SELECTED.0 != 0,
                        corner: control.corner == item.hwndItem,
                        dropdown: control.dropdown.get(),
                        size: crate::layout::px(28, data.dpi.get()),
                    };
                    if let Err(error) = crate::paint_batch::control(item.hDC, item.rcItem, |out| {
                        content
                            .painter
                            .button(out, item.rcItem, control.icon.get(), state)?;
                        let mut buffered = *item;
                        buffered.hDC = out;
                        slots::paint(content, control, &buffered, state, data.dpi.get())
                    }) {
                        eprintln!("neuro-mini-button-paint: {error}");
                    }
                }
            }
            LRESULT(1)
        }
        WM_COMMAND if (wp.0 >> 16) & 0xffff == BN_CLICKED as usize => {
            let id = (wp.0 & 0xffff) as u16;
            let context = data.content.borrow().as_ref().and_then(|content| {
                content
                    .controls
                    .iter()
                    .find(|control| {
                        control.spec.id == (id & 0x7fff)
                            && (wp.0 >> 32 == 0 || wp.0 >> 32 == control.context.get() as usize)
                            && control.visible.get()
                            && (id & 0x8000 == 0 || control.dropdown.get())
                            && (control.hwnd == HWND(lp.0 as *mut _)
                                || control.corner == HWND(lp.0 as *mut _))
                    })
                    .map(|control| control.context.get())
            });
            if let Some(context) =
                context.filter(|_| IsWindowEnabled(HWND(lp.0 as *mut _)).as_bool())
            {
                let _ = PostMessageW(
                    Some(data.owner),
                    data.notify,
                    WPARAM((id & 0x7fff) as usize | ((context as usize) << 32)),
                    LPARAM((id >> 15) as isize),
                );
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            SendMessageW(
                data.owner,
                data.notify,
                Some(WPARAM(0)),
                Some(LPARAM(CLOSE)),
            );
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
