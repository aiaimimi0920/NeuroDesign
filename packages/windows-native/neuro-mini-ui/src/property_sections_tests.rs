//! 连续分组、状态标签和窄计数按钮的真实控件合同；不依赖任何应用业务。
use super::*;
fn action(id: u16) -> (PropertyField, bool) {
    (
        PropertyField {
            id,
            label: "操作",
            icon: None,
            kind: PropertyKind::Action {
                value: String::new(),
                active: false,
                width: 28,
            },
        },
        true,
    )
}
#[test]
fn sections_keep_one_shell_and_state_labels_never_submit_or_take_keyboard_focus() {
    unsafe {
        let (_owner, toolbar) = crate::tests::fixture();
        crate::prepare(toolbar.0).unwrap();
        let panel = PropertyPanel::new(toolbar.0, 125).unwrap();
        let sections = || {
            vec![
                vec![action(1), action(2)],
                vec![
                    (
                        PropertyField {
                            id: 3,
                            label: "状态",
                            icon: None,
                            kind: PropertyKind::Label {
                                value: "-".into(),
                                width: 80,
                            },
                        },
                        true,
                    ),
                    action(4),
                ],
            ]
        };
        panel.update_sections(7, sections()).unwrap();
        let fields = panel.0.data.controls.borrow();
        assert_eq!(fields[0].frame.top, fields[1].frame.top);
        assert_eq!(
            fields[2].frame.top,
            crate::layout::px(36, panel.0.data.dpi.get())
        );
        assert_eq!(fields[2].frame.top, fields[3].frame.top);
        let label = fields[2].hwnd;
        assert_eq!(GetWindowLongW(label, GWL_STYLE) as u32 & WS_TABSTOP.0, 0);
        let handles: Vec<_> = fields.iter().map(|c| c.hwnd).collect();
        drop(fields);
        SendMessageW(
            panel.0.data.viewport.get(),
            WM_COMMAND,
            Some(WPARAM(12)),
            Some(LPARAM(label.0 as isize)),
        );
        assert!(panel.take_edit(3).is_none());
        panel.update_sections(7, sections()).unwrap();
        assert_eq!(
            handles,
            panel
                .0
                .data
                .controls
                .borrow()
                .iter()
                .map(|c| c.hwnd)
                .collect::<Vec<_>>()
        );
        assert!(controls::update(
            &panel.0,
            7,
            sections().into_iter().flatten().collect(),
            vec![3]
        )
        .unwrap()
        .is_none());
    }
}
#[test]
fn panel_has_its_own_bounded_field_budget_without_expanding_toolbar_properties() {
    unsafe {
        let (_owner, toolbar) = crate::tests::fixture();
        crate::prepare(toolbar.0).unwrap();
        let panel = PropertyPanel::new(toolbar.0, 125).unwrap();
        assert!(panel
            .update_with_states(1, (1..=32).map(action).collect())
            .is_ok());
        assert!(panel
            .update_with_states(1, (1..=33).map(action).collect())
            .is_err());
        assert!(
            set_properties(toolbar.0, 1, (1..=17).map(|i| action(i).0).collect(), true).is_err()
        );
        assert!(panel.update_sections(1, vec![vec![]; 9]).is_err());
    }
}
