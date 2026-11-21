//! 字段几何与真实子 HWND 位置；纵向 viewport 裁切整个控件组，不重建草稿或焦点。
use super::*;
pub(super) fn suffix_width(d: &Data, kind: &PropertyKind) -> i32 {
    match kind {
        PropertyKind::Number { suffix, .. } if !suffix.is_empty() => {
            crate::layout::px(suffix.chars().count() as i32 * 10 + 2, d.dpi.get())
        }
        _ => 0,
    }
}
pub(super) unsafe fn arrange(bar: &Bar, limit: i32) -> Result<()> {
    let d = &bar.data;
    let stack = d.stack_width.is_some();
    let padding = if stack { 0 } else { 5 };
    let mut x = padding;
    let mut y = padding;
    let px = |v| crate::layout::px(v, d.dpi.get());
    for c in d.controls.borrow_mut().iter_mut() {
        if d.section_starts.borrow().contains(&c.field.id) && x > padding {
            x = padding;
            y += 36;
        }
        let full = matches!(
            c.field.kind,
            PropertyKind::Number { .. } | PropertyKind::ReadOnly { .. }
        );
        let width = if stack && full {
            limit
        } else {
            controls::width(&c.field.kind)
        };
        if x > padding && x + width > limit + padding {
            x = padding;
            y += 28 + if stack { 2 } else { 4 };
        }
        c.frame = RECT {
            left: px(x),
            top: px(y),
            right: px(x + width),
            bottom: px(y + 28),
        };
        x += width + if stack { 1 } else { 4 };
    }
    let content = px(y + 28 + padding);
    d.content_height.set(content);
    d.width.set(px(d
        .stack_width
        .map_or(if y > 5 { limit + 10 } else { x + 1 }, i32::from)));
    let height = if stack {
        (content + px(8))
            .min(d.max_height.get())
            .min(d.available_height.get())
    } else {
        content
    };
    d.height.set(height);
    Ok(())
}
pub(super) unsafe fn refresh(bar: &Bar) -> Result<()> {
    let d = &bar.data;
    let limit = d.available.get();
    arrange(bar, limit)?;
    if d.stack_width.is_some() {
        if d.content_height.get() > d.height.get() - crate::layout::px(8, d.dpi.get()) {
            arrange(bar, limit - 8)?;
        }
        let pad = crate::layout::px(4, d.dpi.get());
        let rail = if scroll::maximum(d) > 0.0 {
            crate::layout::px(8, d.dpi.get())
        } else {
            0
        };
        SetWindowPos(
            d.viewport.get(),
            None,
            pad,
            pad,
            d.width.get() - pad * 2 - rail,
            d.height.get() - pad * 2,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
    }
    d.scroll_top
        .set(d.scroll_top.get().clamp(0.0, scroll::maximum(d)));
    project(d)?;
    let _ = InvalidateRect(Some(bar.hwnd), None, false);
    Ok(())
}
pub(super) unsafe fn project(d: &Data) -> Result<()> {
    let px = |v| crate::layout::px(v, d.dpi.get());
    let offset = d.scroll_top.get().round() as i32;
    for c in d.controls.borrow().iter() {
        if c.ratio.is_some() {
            ratio::project(c, d.dpi.get(), offset)?;
            continue;
        }
        let r = c.frame;
        let suffix = suffix_width(d, &c.field.kind);
        let (left, top, width, height) = match c.field.kind {
            PropertyKind::Number { .. } => (
                r.left + px(22),
                r.top + px(4),
                r.right - r.left - px(36) - suffix,
                px(20),
            ),
            PropertyKind::ReadOnly { .. } => (
                r.left + px(22),
                r.top + px(4),
                r.right - r.left - px(24),
                px(20),
            ),
            PropertyKind::Text { .. } => (
                r.left + px(3),
                r.top + px(4),
                r.right - r.left - px(6),
                px(20),
            ),
            _ => (r.left, r.top, r.right - r.left, r.bottom - r.top),
        };
        if width <= 0 {
            return Err(Error::new(
                E_BOUNDS,
                "Property field cannot contain its value and unit",
            ));
        }
        SetWindowPos(
            c.hwnd,
            None,
            left,
            top - offset,
            width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
        for (i, h) in c.step.iter().enumerate() {
            if !h.is_invalid() {
                SetWindowPos(
                    *h,
                    None,
                    r.right - px(14),
                    r.top + px(i as i32 * 14) - offset,
                    px(14),
                    px(14),
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )?;
            }
        }
    }
    if !d.viewport.get().is_invalid() {
        let _ = InvalidateRect(Some(d.viewport.get()), None, false);
        if let Ok(root) = GetParent(d.viewport.get()) {
            let _ = InvalidateRect(Some(root), None, false);
        }
    }
    Ok(())
}
