//! 通知的固定主题软件绘制；字体、画刷和 DC 状态均在本次 paint 内回收。
use super::*;
pub(super) fn color(rgb: u32) -> COLORREF {
    COLORREF((rgb & 255) << 16 | (rgb & 0xff00) | rgb >> 16)
}
unsafe fn font(dpi: u32, size: i32, weight: i32) -> HFONT {
    CreateFontW(
        -crate::layout::px(size, dpi),
        0,
        0,
        0,
        weight,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        DEFAULT_PITCH.0 as u32,
        w!("Segoe UI"),
    )
}
pub(super) unsafe fn height(parent: HWND, ui: &Ui, width: i32, dpi: u32) -> Result<i32> {
    let dc = GetDC(Some(parent));
    if dc.is_invalid() {
        return Err(Error::from_win32());
    }
    let font = font(dpi, 10, FW_NORMAL.0 as i32);
    let heading = self::font(dpi, 11, FW_SEMIBOLD.0 as i32);
    if font.is_invalid() || heading.is_invalid() {
        let _ = DeleteObject(HGDIOBJ(font.0));
        let _ = DeleteObject(HGDIOBJ(heading.0));
        ReleaseDC(Some(parent), dc);
        return Err(Error::from_win32());
    }
    let old = SelectObject(dc, HGDIOBJ(heading.0));
    let mut area = RECT {
        right: (width - crate::layout::px(84, dpi)).max(1),
        ..Default::default()
    };
    DrawTextW(
        dc,
        &mut ui.title.clone(),
        &mut area,
        DT_WORDBREAK | DT_CALCRECT | DT_NOPREFIX,
    );
    let title_height = area.bottom;
    ui.title_height.set(title_height);
    area.bottom = 0;
    SelectObject(dc, HGDIOBJ(font.0));
    DrawTextW(
        dc,
        &mut ui.message.clone(),
        &mut area,
        DT_WORDBREAK | DT_CALCRECT | DT_NOPREFIX,
    );
    SelectObject(dc, old);
    let _ = DeleteObject(HGDIOBJ(font.0));
    let _ = DeleteObject(HGDIOBJ(heading.0));
    ReleaseDC(Some(parent), dc);
    Ok(
        (title_height + crate::layout::px(4, dpi) + area.bottom + crate::layout::px(24, dpi))
            .max(crate::layout::px(44, dpi)),
    )
}
pub(super) unsafe fn draw(hwnd: HWND, ui: &Ui) {
    let mut ps = PAINTSTRUCT::default();
    let dc = BeginPaint(hwnd, &mut ps);
    let saved = SaveDC(dc);
    let mut rect = RECT::default();
    let _ = GetClientRect(hwnd, &mut rect);
    let p = crate::Palette::default();
    let brush = CreateSolidBrush(color(p.control));
    FillRect(dc, &rect, brush);
    let _ = DeleteObject(HGDIOBJ(brush.0));
    let brush = CreateSolidBrush(color(crate::paint::mix(p.control, p.signal, 0.38)));
    FrameRect(dc, &rect, brush);
    let _ = DeleteObject(HGDIOBJ(brush.0));
    SetBkMode(dc, TRANSPARENT);
    let dpi = ui.dpi.get();
    let px = |n| crate::layout::px(n, dpi);
    let edge = CreateSolidBrush(color(p.signal));
    FillRect(
        dc,
        &RECT {
            right: px(3),
            ..rect
        },
        edge,
    );
    let _ = DeleteObject(HGDIOBJ(edge.0));
    let content_right = rect.right - if ui.scroll.maximum() > 0.0 { px(12) } else { 0 };
    let top = -(ui.scroll.top.get().round() as i32);
    let clipped = SaveDC(dc);
    IntersectClipRect(dc, 1, 1, content_right.max(1), rect.bottom - 1);
    let title = font(dpi, 11, FW_SEMIBOLD.0 as i32);
    let body = font(dpi, 10, FW_NORMAL.0 as i32);
    let original = SelectObject(dc, HGDIOBJ(title.0));
    SetTextColor(dc, color(p.signal));
    let mut area = RECT {
        left: px(12),
        top: top + px(12),
        right: content_right - px(72),
        bottom: top + px(12) + ui.title_height.get(),
    };
    DrawTextW(
        dc,
        &mut ui.title.clone(),
        &mut area,
        DT_WORDBREAK | DT_NOPREFIX,
    );
    SelectObject(dc, HGDIOBJ(body.0));
    SetTextColor(dc, color(p.muted));
    area = RECT {
        left: px(12),
        top: top + px(16) + ui.title_height.get(),
        right: content_right - px(72),
        bottom: top + ui.measured.get().3 - px(12),
    };
    DrawTextW(
        dc,
        &mut ui.message.clone(),
        &mut area,
        DT_WORDBREAK | DT_NOPREFIX,
    );
    SetTextColor(dc, color(p.text));
    area = RECT {
        left: content_right - px(60),
        top: top + px(12),
        right: content_right - px(12),
        bottom: top + px(32),
    };
    let brush = CreateSolidBrush(color(crate::paint::mix(p.control, p.text, 0.12)));
    FrameRect(dc, &area, brush);
    let _ = DeleteObject(HGDIOBJ(brush.0));
    DrawTextW(
        dc,
        &mut ui.dismiss.clone(),
        &mut area,
        DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_NOPREFIX,
    );
    let _ = RestoreDC(dc, clipped);
    ui.scroll.paint(dc, dpi);
    if windows::Win32::UI::Input::KeyboardAndMouse::GetFocus() == hwnd {
        let mut focus = rect;
        let _ = InflateRect(&mut focus, -px(3), -px(3));
        let _ = DrawFocusRect(dc, &focus);
    }
    SelectObject(dc, original);
    let _ = DeleteObject(HGDIOBJ(title.0));
    let _ = DeleteObject(HGDIOBJ(body.0));
    let _ = RestoreDC(dc, saved);
    let _ = EndPaint(hwnd, &ps);
}
