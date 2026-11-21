//! 只投影可见行，保留部分行像素偏移；滚动不更改选中值，不提交应用命令。
use super::*;
pub(super) unsafe fn project(d: &Data) -> Result<()> {
    // 字体行复用时先完成标签/几何更新，最后发布整个列表，不能逐行露出系统擦底。
    let _hold = crate::paint_batch::Hold::new(d.root.get());
    let top = d.top.get().round() as i32;
    let first = ((top - d.frame.pad).max(0) / d.frame.row) as usize;
    let rows = d.rows.borrow().clone();
    for (slot, hwnd) in rows.into_iter().enumerate() {
        let index = first + slot;
        if index >= d.items.len() {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            let _ = ShowWindow(hwnd, SW_HIDE);
            continue;
        }
        if GetWindowLongPtrW(hwnd, GWLP_USERDATA) != index as isize + 1 {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, index as isize + 1);
            SetWindowTextW(hwnd, &HSTRING::from(&d.items[index]))?;
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        let _ = EnableWindow(hwnd, d.enabled[index]);
        SetWindowPos(
            hwnd,
            None,
            0,
            d.frame.pad + index as i32 * d.frame.row - top,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )?;
    }
    let _ = InvalidateRect(Some(d.root.get()), None, false);
    Ok(())
}
pub(super) unsafe fn set(d: &Data, top: f64) -> Result<()> {
    let top = top.clamp(0.0, d.frame.maximum());
    if d.top.replace(top) != top {
        d.pressed.set(None);
        d.hover.set(None);
        project(d)?;
    }
    Ok(())
}
pub(super) unsafe fn wheel(d: &Data, delta: i16) -> Result<()> {
    set(d, d.top.get() - delta as f64 * d.frame.row as f64 / 120.0)
}
pub(super) unsafe fn reveal(d: &Data, index: usize) -> Result<()> {
    let y = d.frame.pad + index as i32 * d.frame.row;
    let top = d.top.get();
    if (y as f64) < top {
        set(d, y as f64)
    } else if (y + d.frame.row) as f64 > top + d.frame.height() as f64 {
        set(d, (y + d.frame.row - d.frame.height()) as f64)
    } else {
        Ok(())
    }
}
pub(super) unsafe fn invalidate_rows(d: &Data) {
    let rows = d.rows.borrow().clone();
    for hwnd in rows {
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}
pub(super) unsafe fn tab(d: &Data, backwards: bool) -> Result<()> {
    let len = d.items.len() + usize::from(d.frame.maximum() > 0.0);
    let mut next = match d.focus.get() {
        None => {
            if backwards {
                len - 1
            } else {
                0
            }
        }
        Some(i) => {
            if backwards {
                (i + len - 1) % len
            } else {
                (i + 1) % len
            }
        }
    };
    for _ in 0..len {
        if next == d.items.len() || d.enabled[next] {
            break;
        }
        next = if backwards {
            (next + len - 1) % len
        } else {
            (next + 1) % len
        };
    }
    d.focus
        .set((next == d.items.len() || d.enabled[next]).then_some(next));
    if next < d.items.len() {
        reveal(d, next)?;
    }
    invalidate_rows(d);
    let _ = InvalidateRect(Some(d.root.get()), None, false);
    Ok(())
}
pub(super) unsafe fn key(d: &Data, key: u16) -> Result<()> {
    if d.focus.get() != Some(d.items.len()) {
        return Ok(());
    }
    let top = d.top.get();
    let next = match key {
        k if k == VK_DOWN.0 => top + d.frame.row as f64,
        k if k == VK_UP.0 => top - d.frame.row as f64,
        k if k == VK_NEXT.0 => top + d.frame.height() as f64,
        k if k == VK_PRIOR.0 => top - d.frame.height() as f64,
        k if k == VK_HOME.0 => 0.0,
        k if k == VK_END.0 => d.frame.maximum(),
        _ => return Ok(()),
    };
    set(d, next)
}
pub(super) unsafe fn down(d: &Data, p: POINT) -> Result<()> {
    if d.frame.maximum() > 0.0 && geometry::inside(d.frame.track(), p) {
        d.focus.set(Some(d.items.len()));
        let thumb = d.frame.thumb(d.top.get());
        let travel = d.frame.height() - (thumb.bottom - thumb.top);
        if geometry::inside(thumb, p) {
            d.drag.set(Some(Drag {
                y: p.y,
                top: d.top.get(),
                maximum: d.frame.maximum(),
                travel,
            }));
        } else if travel > 0 {
            set(
                d,
                (p.y - d.frame.track().top - (thumb.bottom - thumb.top) / 2) as f64
                    * d.frame.maximum()
                    / travel as f64,
            )?;
        }
        d.pressed.set(None);
    } else {
        let hit = d
            .frame
            .hit(p, d.top.get(), d.items.len())
            .filter(|i| d.enabled[*i]);
        d.pressed.set(hit);
        d.focus.set(hit);
    }
    invalidate_rows(d);
    Ok(())
}
pub(super) unsafe fn motion(d: &Data, p: POINT) -> Result<()> {
    if let Some(drag) = d.drag.get() {
        if drag.travel > 0 {
            set(
                d,
                drag.top + (p.y - drag.y) as f64 * drag.maximum / drag.travel as f64,
            )?;
        }
    } else {
        let hit = d
            .frame
            .hit(p, d.top.get(), d.items.len())
            .filter(|i| d.enabled[*i]);
        if d.hover.replace(hit) != hit {
            invalidate_rows(d);
        }
    }
    Ok(())
}
