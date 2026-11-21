//! 同一通知栈的测量、裁剪与定位；保留稳定子窗口，空隙不覆盖或抢走图片输入。
use super::*;
struct Placing<'a>(&'a Cell<bool>);
impl Drop for Placing<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}
unsafe fn cards(data: &Data, width: i32, height: i32, dpi: u32) -> Result<Vec<(HWND, i32)>> {
    let handles: Vec<_> = data
        .entries
        .borrow()
        .iter()
        .filter_map(|w| w.upgrade())
        .map(|live| live.get())
        .filter(|hwnd| !hwnd.is_invalid())
        .collect();
    handles
        .into_iter()
        .map(|hwnd| {
            let ui = &*(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Ui);
            let limit = height.min(crate::layout::px(140, dpi));
            let content = super::super::measure(hwnd, ui, width, dpi, limit)?;
            Ok((hwnd, content.min(limit)))
        })
        .collect()
}
fn total(cards: &[(HWND, i32)], gap: i32) -> i32 {
    cards.iter().map(|(_, height)| *height).sum::<i32>()
        + gap * cards.len().saturating_sub(1) as i32
}
unsafe fn position(hwnd: HWND, x: i32, y: i32, width: i32, height: i32, child: bool) -> Result<()> {
    let mut old = RECT::default();
    GetWindowRect(hwnd, &mut old)?;
    if child {
        let mut point = POINT {
            x: old.left,
            y: old.top,
        };
        if !ScreenToClient(GetParent(hwnd)?, &mut point).as_bool() {
            return Err(Error::from_win32());
        }
        old.left = point.x;
        old.top = point.y;
        let mut size = RECT::default();
        GetClientRect(hwnd, &mut size)?;
        old.right = old.left + size.right;
        old.bottom = old.top + size.bottom;
    }
    if (
        old.left,
        old.top,
        old.right - old.left,
        old.bottom - old.top,
    ) != (x, y, width, height)
    {
        SetWindowPos(
            hwnd,
            None,
            x,
            y,
            width,
            height,
            SWP_NOACTIVATE | SWP_NOZORDER,
        )?;
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
    Ok(())
}
unsafe fn region(root: HWND, data: &Data, areas: Vec<RECT>) -> Result<()> {
    if *data.region.borrow() == areas {
        return Ok(());
    }
    let region = CreateRectRgn(0, 0, 0, 0);
    if region.is_invalid() {
        return Err(Error::from_win32());
    }
    for r in &areas {
        let part = CreateRectRgn(r.left, r.top, r.right, r.bottom);
        if part.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(region.0));
            return Err(Error::from_win32());
        }
        let result = CombineRgn(Some(region), Some(region), Some(part), RGN_OR);
        let _ = DeleteObject(HGDIOBJ(part.0));
        if result == RGN_ERROR {
            let _ = DeleteObject(HGDIOBJ(region.0));
            return Err(Error::from_win32());
        }
    }
    if SetWindowRgn(root, Some(region), true) == 0 {
        let _ = DeleteObject(HGDIOBJ(region.0));
        return Err(Error::from_win32());
    }
    *data.region.borrow_mut() = areas;
    Ok(())
}
pub(super) unsafe fn place(data: &Data) -> Result<()> {
    let root = data.root.get();
    if root.is_invalid() || data.placing.replace(true) {
        return Ok(());
    }
    let _placing = Placing(&data.placing);
    data.entries
        .borrow_mut()
        .retain(|w| w.upgrade().is_some_and(|live| !live.get().is_invalid()));
    let mut rect = RECT::default();
    GetClientRect(data.parent, &mut rect)?;
    let dpi = GetDpiForWindow(data.parent).max(96);
    let px = |n| crate::layout::px(n, dpi);
    let width = px(280).min((rect.right - px(16)).max(1));
    let available = (rect.bottom - px(16)).max(1);
    let gap = px(8);
    let mut entries = cards(data, width, available, dpi)?;
    let overflow = total(&entries, gap) > available;
    let card_width = if overflow {
        (width - px(12)).max(1)
    } else {
        width
    };
    if overflow {
        entries = cards(data, card_width, available, dpi)?;
    }
    let content = total(&entries, gap);
    let height = content.min(available).max(1);
    if data.dpi.replace(dpi) != dpi {
        data.scroll.top.set(0.0);
    }
    data.scroll.configure(width, height, content);
    let frame = (width, height, content, data.scroll.top.get().to_bits(), dpi);
    let changed = data.frame.replace(frame) != frame;
    let mut origin = POINT::default();
    if !ClientToScreen(data.parent, &mut origin).as_bool() {
        return Err(Error::from_win32());
    }
    position(
        root,
        origin.x + (rect.right - px(8) - width).max(0),
        origin.y + px(8).min((rect.bottom - height).max(0)),
        width,
        height,
        false,
    )?;
    let mut affinity = 0;
    GetWindowDisplayAffinity(data.parent, &mut affinity)?;
    if data.affinity.get() != affinity {
        let _ = ShowWindow(root, SW_HIDE);
        SetWindowDisplayAffinity(root, WINDOW_DISPLAY_AFFINITY(affinity))?;
        data.affinity.set(affinity);
    }
    let mut areas = Vec::with_capacity(entries.len() + 1);
    let mut y = -(data.scroll.top.get().round() as i32);
    let visible = IsWindowVisible(data.parent).as_bool() && !entries.is_empty();
    for (hwnd, height) in entries {
        let ui = &*(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Ui);
        let content = super::super::measure(hwnd, ui, card_width, dpi, available.min(px(140)))?;
        let old_dpi = ui.dpi.replace(dpi);
        let old_top = ui.scroll.top.get();
        ui.scroll.configure(card_width, height, content);
        position(hwnd, 0, y, card_width, height, true)?;
        if old_dpi != dpi || old_top != ui.scroll.top.get() {
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        if !visible {
            super::super::cancel(hwnd, ui);
        }
        areas.push(RECT {
            left: 0,
            top: y.max(0),
            right: card_width,
            bottom: (y + height).min(data.scroll.track(dpi).bottom),
        });
        if GetWindowLongW(hwnd, GWL_STYLE) as u32 & WS_VISIBLE.0 == 0 {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        y += height + gap;
    }
    areas.retain(|r| r.top < r.bottom);
    if overflow {
        areas.push(data.scroll.track(dpi));
    }
    region(root, data, areas)?;
    if changed {
        let _ = InvalidateRect(Some(root), None, false);
    }
    if !visible {
        data.scroll.cancel(root);
    }
    if IsWindowVisible(root).as_bool() != visible {
        let _ = ShowWindow(root, if visible { SW_SHOWNOACTIVATE } else { SW_HIDE });
    }
    Ok(())
}
pub(super) unsafe fn reveal(data: &Data, hwnd: HWND) {
    if hwnd.is_invalid() {
        return;
    }
    let mut r = RECT::default();
    if GetWindowRect(hwnd, &mut r).is_err() {
        return;
    }
    let mut point = POINT {
        x: r.left,
        y: r.top,
    };
    if !ScreenToClient(data.root.get(), &mut point).as_bool() {
        return;
    }
    let height = data.scroll.track(data.dpi.get()).bottom;
    let delta = if point.y < 0 {
        point.y
    } else {
        (point.y + r.bottom - r.top - height).max(0)
    };
    if data.scroll.set(data.scroll.top.get() + delta as f64) {
        if let Err(error) = place(data) {
            fail(data.root.get(), error);
        }
    }
}
