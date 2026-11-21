//! child 窗口和即时局部滚动；按钮动作仅投递 index/context，应用负责验证和执行。
use super::*;
pub(super) unsafe fn place(hwnd: HWND, data: &Data) -> Result<()> {
    let layout = data.layout.borrow().clone();
    SetWindowPos(
        hwnd,
        None,
        0,
        0,
        layout.width,
        layout.height,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
    )?;
    for (button, frame) in data.buttons.iter().zip(&layout.cards) {
        SetWindowPos(
            *button,
            None,
            frame.x,
            frame.y,
            frame.width,
            frame.height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
    }
    InvalidateRect(Some(hwnd), None, false).ok()
}
unsafe fn scroll(hwnd: HWND, data: &Data, offset: f64) {
    let old = data.layout.borrow().clone();
    let next = PreviewGridLayout::new(data.buttons.len(), old.dpi, offset);
    if next != old {
        *data.layout.borrow_mut() = next;
        if let Err(error) = place(hwnd, data) {
            eprintln!("neuro-mini-preview-scroll: {error}");
        }
    }
}
pub(super) unsafe fn reveal(hwnd: HWND, data: &Data, focus: HWND) {
    let Some(index) = data.buttons.iter().position(|h| *h == focus) else {
        return;
    };
    let l = data.layout.borrow().clone();
    let f = l.cards[index];
    let delta = if f.y < 0 {
        f.y
    } else {
        (f.y + f.height - l.height).max(0)
    };
    scroll(hwnd, data, l.offset + delta as f64);
}
pub(super) unsafe extern "system" fn procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Data;
    if msg == WM_NCDESTROY {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        if !ptr.is_null() {
            let data = Box::from_raw(ptr);
            data.lifetime.set(false);
            if GetCapture() == hwnd {
                let _ = ReleaseCapture();
            }
        }
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let data = &*ptr;
    match msg {
        WM_CLOSE => return LRESULT(0),
        WM_ERASEBKGND => return LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut ps);
            paint::shell(dc, data);
            let _ = EndPaint(hwnd, &ps);
            return LRESULT(0);
        }
        WM_DRAWITEM if lp.0 != 0 => {
            paint::card(data, &*(lp.0 as *const DRAWITEMSTRUCT));
            return LRESULT(1);
        }
        WM_COMMAND if wp.0 >> 16 == BN_CLICKED as usize => {
            if let Some(index) = data.buttons.iter().position(|h| h.0 as isize == lp.0) {
                if IsWindowVisible(hwnd).as_bool()
                    && IsWindowEnabled(data.buttons[index]).as_bool()
                    && data.context.get() != 0
                {
                    let _ = PostMessageW(
                        Some(data.parent),
                        data.notify,
                        WPARAM(index),
                        LPARAM(data.context.get() as isize),
                    );
                }
            }
            return LRESULT(0);
        }
        WM_MOUSEWHEEL => {
            let offset = data.layout.borrow().offset;
            scroll(
                hwnd,
                data,
                offset - ((wp.0 >> 16) as u16 as i16) as f64 * 0.4,
            );
            return LRESULT(0);
        }
        WM_LBUTTONDOWN => {
            let y = (lp.0 >> 16) as u16 as i16 as i32;
            let x = lp.0 as u16 as i16 as i32;
            let l = data.layout.borrow().clone();
            if let (Some(track), Some(thumb)) = (l.track(), l.thumb()) {
                if x >= track.x
                    && x < track.x + track.width
                    && y >= track.y
                    && y < track.y + track.height
                {
                    let maximum = (l.content - l.height) as f64;
                    if y >= thumb.y && y < thumb.y + thumb.height {
                        data.drag.set(Some((
                            y,
                            l.offset,
                            maximum / (track.height - thumb.height).max(1) as f64,
                        )));
                        SetCapture(hwnd);
                    } else {
                        scroll(
                            hwnd,
                            data,
                            (y - track.y) as f64 / track.height as f64 * maximum,
                        );
                    }
                    return LRESULT(0);
                }
            }
        }
        WM_MOUSEMOVE if GetCapture() == hwnd => {
            if let Some((start, top, ratio)) = data.drag.get() {
                scroll(
                    hwnd,
                    data,
                    top + ((lp.0 >> 16) as u16 as i16 as i32 - start) as f64 * ratio,
                );
            }
            return LRESULT(0);
        }
        WM_LBUTTONUP | WM_CANCELMODE | WM_SHOWWINDOW if msg != WM_SHOWWINDOW || wp.0 == 0 => {
            data.drag.set(None);
            if GetCapture() == hwnd {
                let _ = ReleaseCapture();
            }
        }
        WM_CAPTURECHANGED => data.drag.set(None),
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}
