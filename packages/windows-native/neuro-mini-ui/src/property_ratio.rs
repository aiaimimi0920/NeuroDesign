//! 比例组的两个真实 EDIT 与外侧步进；值经原属性通知提交，不新增窗口 owner 或业务状态。
use super::*;
pub(super) struct Inputs {
    pub values: [HWND; 2],
    steps: [HWND; 4],
}
impl Inputs {
    pub fn handles(&self) -> impl Iterator<Item = HWND> {
        self.values.into_iter().chain(self.steps)
    }
}
pub(super) fn parts(value: &str) -> Option<[&str; 2]> {
    if value.len() > 32 {
        return None;
    }
    let (a, b) = value.split_once(':')?;
    for part in [a, b] {
        let mut decimal = part.split('.');
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        if !digits(decimal.next()?)
            || decimal.next().is_some_and(|s| !digits(s))
            || decimal.next().is_some()
        {
            return None;
        }
        let n = part.parse::<f64>().ok()?;
        if !n.is_finite() || !(0.01..=10000.0).contains(&n) {
            return None;
        }
    }
    Some([a, b])
}
pub(super) unsafe fn create(parent: HWND, c: &mut Control, font: HFONT) -> Result<()> {
    let r = c.ratio.insert(Inputs {
        values: [HWND::default(); 2],
        steps: [HWND::default(); 4],
    });
    // 高半区 ID 与原字段的四槽 ID 不重叠；消息按 HWND 身份匹配，不暴露子槽给宿主。
    let base = 0x8000 + c.field.id as usize * 8;
    for i in 0..2 {
        r.values[i] = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("EDIT"),
            &HSTRING::from(if i == 0 {
                "比例宽度"
            } else {
                "比例高度"
            }),
            WS_CHILD | WS_TABSTOP | WINDOW_STYLE((ES_CENTER | ES_AUTOHSCROLL | ES_NUMBER) as u32),
            0,
            0,
            1,
            1,
            Some(parent),
            Some(HMENU((base + i) as *mut _)),
            None,
            None,
        )?;
        edit::install(r.values[i])?;
        SendMessageW(
            r.values[i],
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(LPARAM(0)),
        );
        SendMessageW(r.values[i], EM_SETLIMITTEXT, Some(WPARAM(3)), None);
    }
    for i in 0..4 {
        r.steps[i] = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            &HSTRING::from(format!(
                "{}比例{}",
                if i % 2 == 0 { "增加" } else { "减少" },
                if i < 2 { "宽度" } else { "高度" }
            )),
            WS_CHILD | WINDOW_STYLE(BS_OWNERDRAW as u32),
            0,
            0,
            1,
            1,
            Some(parent),
            Some(HMENU((base + 2 + i) as *mut _)),
            None,
            None,
        )?;
    }
    Ok(())
}
pub(super) unsafe fn set_value(c: &Control) -> Result<()> {
    let (Some(r), PropertyKind::Ratio { value, .. }) = (&c.ratio, &c.field.kind) else {
        return Ok(());
    };
    let values = parts(value).expect("validated ratio field");
    for (h, value) in r.values.iter().zip(values) {
        let mut old = [0u16; 65];
        let n = GetWindowTextW(*h, &mut old) as usize;
        if String::from_utf16_lossy(&old[..n]) != value {
            SetWindowTextW(*h, &HSTRING::from(value))?;
        }
    }
    Ok(())
}
pub(super) unsafe fn project(c: &Control, dpi: u32, offset: i32) -> Result<()> {
    let Some(r) = &c.ratio else { return Ok(()) };
    let px = |v| crate::layout::px(v, dpi);
    let frame = c.frame;
    for (h, x, y, w, height) in std::iter::once((c.hwnd, 0, 0, 15, 28))
        .chain(
            r.values
                .iter()
                .enumerate()
                .map(|(i, h)| (*h, if i == 0 { 28 } else { 60 }, 4, 26, 20)),
        )
        .chain(
            r.steps
                .iter()
                .enumerate()
                .map(|(i, h)| (*h, if i < 2 { 15 } else { 86 }, (i % 2) as i32 * 14, 13, 14)),
        )
    {
        SetWindowPos(
            h,
            None,
            frame.left + px(x),
            frame.top + px(y) - offset,
            px(w),
            px(height),
            SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
    }
    Ok(())
}
pub(super) unsafe fn command(parent: HWND, d: &Data, child: HWND, code: usize) -> bool {
    if code == BN_CLICKED as usize {
        let toggle = {
            let mut controls = d.controls.borrow_mut();
            controls
                .iter_mut()
                .find(|c| c.hwnd == child && c.ratio.is_some())
                .map(|c| {
                    let PropertyKind::Ratio { value, active } = &mut c.field.kind else {
                        unreachable!()
                    };
                    *active = !*active;
                    let first = c.pending.is_none();
                    c.pending = Some(PropertyEdit {
                        context: d.context.get(),
                        id: c.field.id,
                        value: Some(value.clone()),
                        active: Some(*active),
                    });
                    (c.field.id, first)
                })
        };
        if let Some((id, first)) = toggle {
            let _ = RedrawWindow(Some(parent), None, None, RDW_INVALIDATE | RDW_ALLCHILDREN);
            if first {
                edit::notify(d, id);
            }
            return true;
        }
    }
    let target = d.controls.borrow().iter().find_map(|c| {
        let r = c.ratio.as_ref()?;
        r.steps
            .iter()
            .position(|h| *h == child)
            .map(|i| (r.values[i / 2], if i % 2 == 0 { 1 } else { -1 }))
    });
    if code == BN_CLICKED as usize {
        if let Some((value, step)) = target {
            edit::commit(parent, value, step, false);
            return true;
        }
    }
    false
}
pub(super) unsafe fn commit(d: &Data, child: HWND, step: i32, cancel: bool) -> bool {
    let mut controls = d.controls.borrow_mut();
    let Some(c) = controls
        .iter_mut()
        .find(|c| c.ratio.as_ref().is_some_and(|r| r.values.contains(&child)))
    else {
        return false;
    };
    let r = c.ratio.as_ref().unwrap();
    let mut texts = Vec::with_capacity(2);
    for h in r.values {
        let mut raw = [0u16; 65];
        let n = GetWindowTextW(h, &mut raw) as usize;
        texts.push(String::from_utf16_lossy(&raw[..n]));
    }
    let PropertyKind::Ratio { value, active } = &mut c.field.kind else {
        unreachable!()
    };
    let original = parts(value).expect("validated ratio field");
    if step != 0 && !cancel {
        let i = usize::from(r.values[1] == child);
        let n = texts[i]
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .unwrap_or_else(|| original[i].parse().unwrap());
        let min = if n.fract() == 0.0 { 1.0 } else { 0.01 };
        texts[i] = (n + step as f64).clamp(min, 999.0).to_string();
    }
    let next = texts.join(":");
    let changed = !cancel && (c.dirty || step != 0) && parts(&next).is_some();
    if changed {
        *value = next;
        *active = true;
    }
    c.dirty = false;
    let first = c.pending.is_none();
    if changed {
        c.pending = Some(PropertyEdit {
            context: d.context.get(),
            id: c.field.id,
            value: Some(value.clone()),
            active: Some(*active),
        });
    }
    let id = c.field.id;
    // SetWindowText 同步通知父窗；先释放控件借用，再沿原 updating 门抑制 EN_CHANGE。
    drop(controls);
    d.updating.set(true);
    if let Some(c) = d.controls.borrow().iter().find(|c| c.field.id == id) {
        if let Err(e) = set_value(c) {
            eprintln!("neuro-mini-ratio-value: {e}");
        }
    }
    d.updating.set(false);
    if changed {
        if let Ok(parent) = GetParent(child) {
            let _ = RedrawWindow(Some(parent), None, None, RDW_INVALIDATE | RDW_ALLCHILDREN);
        }
    }
    if changed && first {
        edit::notify(d, id);
    }
    true
}
