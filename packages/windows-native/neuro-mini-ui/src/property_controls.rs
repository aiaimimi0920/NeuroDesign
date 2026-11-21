//! 属性控件按 schema 换代，值刷新保留同目标焦点和未提交草稿。
use super::*;
fn same(a: &PropertyField, b: &PropertyField) -> bool {
    a.id == b.id && std::mem::discriminant(&a.kind) == std::mem::discriminant(&b.kind)
}
pub(super) fn width(kind: &PropertyKind) -> i32 {
    match kind {
        PropertyKind::Number { suffix, .. } => {
            72 + if suffix.is_empty() {
                0
            } else {
                suffix.chars().count() as i32 * 10 + 2
            }
        }
        PropertyKind::ReadOnly { .. } => 72,
        PropertyKind::Text { width, .. } | PropertyKind::Label { width, .. } => *width as i32,
        PropertyKind::Color { .. } => 28,
        PropertyKind::Ratio { .. } => 103,
        PropertyKind::Action { width, .. } | PropertyKind::Dropdown { width, .. } => *width as i32,
    }
}
pub(super) unsafe fn update(
    bar: &Bar,
    context: u64,
    fields: Vec<(PropertyField, bool)>,
    sections: Vec<u16>,
) -> Result<Option<crate::paint_batch::Hold>> {
    let d = &bar.data;
    let dpi = GetDpiForWindow(d.toolbar).max(96);
    let mut rect = RECT::default();
    GetWindowRect(d.toolbar, &mut rect)?;
    let mut monitor = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(
        MonitorFromRect(&rect, MONITOR_DEFAULTTONEAREST),
        &mut monitor,
    )
    .as_bool()
    {
        return Err(Error::from_win32());
    }
    let limit = d.stack_width.map_or(
        (monitor.rcWork.right - monitor.rcWork.left) * 96 / dpi as i32 - 10,
        |w| i32::from(w) - 8,
    );
    let available_height = monitor.rcWork.bottom - monitor.rcWork.top;
    if *d.section_starts.borrow() == sections
        && d.dpi.get() == dpi
        && d.available.get() == limit
        && d.available_height.get() == available_height
    {
        let old = d.controls.borrow();
        if old.len() == fields.len()
            && old.iter().zip(&fields).all(|(a, (b, enabled))| {
                a.field == *b
                    && a.handles()
                        .all(|h| IsWindowEnabled(h).as_bool() == *enabled)
            })
        {
            drop(old);
            // 业务凭证更新不等于外观变化；退休过期编辑，但不重排/重绘整栏。
            if d.context.replace(context) != context {
                d.updating.set(true);
                let result: Result<()> = (|| {
                    for control in d.controls.borrow_mut().iter_mut() {
                        let edited = control.dirty || control.pending.is_some();
                        control.dirty = false;
                        control.pending = None;
                        if edited {
                            set_value(control)?;
                        }
                    }
                    Ok(())
                })();
                d.updating.set(false);
                result?;
            }
            return Ok(None);
        }
    }
    let redraw = crate::paint_batch::Hold::new(bar.hwnd);
    d.updating.set(true);
    let result = (|| {
        let changed = d.context.replace(context) != context;
        d.section_starts.replace(sections);
        let schema = {
            let old = d.controls.borrow();
            old.len() != fields.len()
                || old
                    .iter()
                    .zip(&fields)
                    .any(|(a, (b, _))| !same(&a.field, b))
        };
        if schema {
            // 目标凭证换代只丢草稿；同一字段结构的值提交不应把面板卷回顶部。
            d.scroll_top.set(0.0);
            let mut created = Vec::new();
            for (field, _) in &fields {
                if !d.controls.borrow().iter().any(|c| same(&c.field, field)) {
                    match create(bar, field.clone()) {
                        Ok(c) => created.push(c),
                        Err(e) => {
                            destroy(bar, &created);
                            return Err(e);
                        }
                    }
                }
            }
            // 只替换真正新增/移除的字段；椭圆/矩形切换时其余 EDIT 和按钮不重建。
            let mut old = d.controls.take();
            let next = fields
                .iter()
                .map(|(f, _)| {
                    if let Some(i) = old.iter().position(|c| same(&c.field, f)) {
                        old.remove(i)
                    } else {
                        let i = created.iter().position(|c| same(&c.field, f)).unwrap();
                        created.remove(i)
                    }
                })
                .collect();
            d.controls.replace(next);
            destroy(bar, &old);
        }
        {
            for (c, (field, enabled)) in d.controls.borrow_mut().iter_mut().zip(fields) {
                if changed {
                    c.dirty = false;
                    c.pending = None;
                }
                let different = c.field != field;
                if c.field.label != field.label {
                    let label = HSTRING::from(field.label);
                    if let Some(tip) = d.tip.borrow().as_ref() {
                        tip.update(bar.hwnd, c.hwnd, &label);
                    }
                    c.label = label;
                }
                if c.pending.is_none() {
                    c.field = field;
                }
                if !c.dirty && c.pending.is_none() {
                    set_value(c)?;
                }
                if different {
                    let _ = InvalidateRect(Some(c.hwnd), None, false);
                }
                // 准入与值在同一冻结帧中提交，不能先启用所有字段再发布禁用状态。
                for h in c.handles() {
                    if IsWindowEnabled(h).as_bool() != enabled {
                        let _ = EnableWindow(h, enabled);
                    }
                }
            }
        }
        if d.dpi.get() != dpi {
            let font = font(dpi, 12)?;
            let small = match self::font(dpi, 10) {
                Ok(f) => f,
                Err(e) => {
                    let _ = DeleteObject(HGDIOBJ(font.0));
                    return Err(e);
                }
            };
            let old = d.font.replace(font);
            let old_small = d.small_font.replace(small);
            for h in d.controls.borrow().iter().flat_map(Control::handles) {
                SendMessageW(
                    h,
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
            let _ = DeleteObject(HGDIOBJ(old.0));
            let _ = DeleteObject(HGDIOBJ(old_small.0));
        }
        d.dpi.set(dpi);
        d.available.set(limit);
        d.available_height.set(available_height);
        layout::refresh(bar)?;
        for c in d.controls.borrow().iter() {
            for h in c.handles() {
                // 创建阶段保持隐藏；字段、字体、值、准入和最终几何就绪后才可见。
                let _ = ShowWindow(h, SW_SHOWNOACTIVATE);
            }
        }
        let _ = InvalidateRect(Some(bar.hwnd), None, false);
        Ok(())
    })();
    d.updating.set(false);
    result.map(|()| Some(redraw))
}
unsafe fn create(bar: &Bar, field: PropertyField) -> Result<Control> {
    let number = matches!(field.kind, PropertyKind::Number { .. });
    let readonly = matches!(field.kind, PropertyKind::ReadOnly { .. });
    let input = number || readonly || matches!(field.kind, PropertyKind::Text { .. });
    let label = matches!(field.kind, PropertyKind::Label { .. });
    let id = field.id as usize * 4;
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        if input {
            w!("EDIT")
        } else if label {
            w!("STATIC")
        } else {
            w!("BUTTON")
        },
        &HSTRING::from(field.label),
        WS_CHILD
            | if label {
                WINDOW_STYLE::default()
            } else {
                WS_TABSTOP
            }
            | WINDOW_STYLE(if input {
                (if bar.data.stack_width.is_some() {
                    ES_RIGHT
                } else {
                    ES_CENTER
                }) as u32
                    | ES_AUTOHSCROLL as u32
                    | if number {
                        ES_NUMBER as u32
                    } else if readonly {
                        ES_READONLY as u32
                    } else {
                        0
                    }
            } else if label {
                windows::Win32::System::SystemServices::SS_OWNERDRAW.0
            } else {
                BS_OWNERDRAW as u32
            }),
        0,
        0,
        1,
        1,
        Some(parent(bar)),
        Some(HMENU(id as *mut _)),
        None,
        None,
    )?;
    SendMessageW(
        hwnd,
        WM_SETFONT,
        Some(WPARAM(bar.data.font.get().0 as usize)),
        Some(LPARAM(0)),
    );
    let mut c = Control {
        label: HSTRING::from(field.label),
        field,
        hwnd,
        step: [HWND::default(); 2],
        ratio: None,
        frame: RECT::default(),
        dirty: false,
        pending: None,
        expanded: false,
    };
    let result = (|| {
        if matches!(c.field.kind, PropertyKind::Ratio { .. }) {
            ratio::create(parent(bar), &mut c, bar.data.font.get())?;
        }
        if bar.data.stack_width.is_some() && !label {
            focus::install(hwnd)?;
        }
        if input && !readonly {
            edit::install(hwnd)?;
            SendMessageW(
                hwnd,
                EM_SETLIMITTEXT,
                Some(WPARAM(
                    if let PropertyKind::Number { min, max, .. } = c.field.kind {
                        min.to_string().len().max(max.to_string().len())
                    } else {
                        32
                    },
                )),
                None,
            );
        }
        if number {
            for i in 0..2 {
                c.step[i] = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BUTTON"),
                    &HSTRING::from(format!(
                        "{}{}",
                        if i == 0 { "增加" } else { "减少" },
                        c.field.label
                    )),
                    WS_CHILD | WINDOW_STYLE(BS_OWNERDRAW as u32),
                    0,
                    0,
                    1,
                    1,
                    Some(parent(bar)),
                    Some(HMENU((id + i + 1) as *mut _)),
                    None,
                    None,
                )?;
                if bar.data.stack_width.is_some() {
                    focus::install(c.step[i])?;
                }
            }
        }
        if let Some(tip) = bar.data.tip.borrow().as_ref() {
            tip.add(bar.hwnd, hwnd, &c.label)?;
        }
        set_value(&c)
    })();
    if let Err(e) = result {
        destroy(bar, &[c]);
        return Err(e);
    }
    Ok(c)
}
fn parent(bar: &Bar) -> HWND {
    if bar.data.stack_width.is_some() {
        bar.data.viewport.get()
    } else {
        bar.hwnd
    }
}
pub(super) unsafe fn font(dpi: u32, size: i32) -> Result<HFONT> {
    let font = CreateFontW(
        -crate::layout::px(size, dpi),
        0,
        0,
        0,
        FW_NORMAL.0 as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        0,
        w!("Segoe UI"),
    );
    if font.is_invalid() {
        Err(Error::from_win32())
    } else {
        Ok(font)
    }
}
unsafe fn destroy(bar: &Bar, controls: &[Control]) {
    for c in controls {
        if let Some(tip) = bar.data.tip.borrow().as_ref() {
            tip.remove(bar.hwnd, c.hwnd, &c.label);
        }
        for h in c.handles() {
            let _ = DestroyWindow(h);
        }
    }
}
unsafe fn set_value(c: &Control) -> Result<()> {
    if c.ratio.is_some() {
        return ratio::set_value(c);
    }
    let value = match &c.field.kind {
        PropertyKind::Number { value, .. } => value.to_string(),
        PropertyKind::Text { value, .. } | PropertyKind::ReadOnly { value } => value.clone(),
        _ => return Ok(()),
    };
    let mut old = [0u16; 65];
    let n = GetWindowTextW(c.hwnd, &mut old) as usize;
    if String::from_utf16_lossy(&old[..n]) != value {
        SetWindowTextW(c.hwnd, &HSTRING::from(value))?;
    }
    Ok(())
}
