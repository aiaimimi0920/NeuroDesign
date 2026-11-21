//! 故障注入只在测试中；生产与测试经过相同的事务分配和回滚路径。
use super::*;
#[test]
fn partial_failure_and_reentrant_show_do_not_publish_controls_or_leak_handles() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        let mut partial = None;
        let result = prepare_with(bar.0, |index| {
            if index == 1 {
                partial = GetDlgItem(Some(bar.0), 1).ok();
                let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
                assert!(!IsWindowVisible(bar.0).as_bool());
                return Err(Error::new(E_OUTOFMEMORY, "injected allocation failure"));
            }
            Ok(())
        });
        assert!(result.is_err());
        assert!(!crate::ready(bar.0));
        assert!(!IsWindow(Some(partial.unwrap())).as_bool());
        assert!(GetDlgItem(Some(bar.0), 1).is_err());
        prepare(bar.0).unwrap();
        assert!(crate::ready(bar.0));
        let button = GetDlgItem(Some(bar.0), 1).unwrap();
        prepare(bar.0).unwrap();
        assert_eq!(button, GetDlgItem(Some(bar.0), 1).unwrap());
    }
}
