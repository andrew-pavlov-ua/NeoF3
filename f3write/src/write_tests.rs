// f3-write/tests/write_tests.rs

use crate::*;

use f3core::flow::Flow;

#[test]
fn integration_create_one_sector() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().to_str().unwrap();

    let mut flow = Flow::new(512, -1, false);
    let stop = create_and_fill_file(p, 1, 512, false, &mut flow);
    assert!(stop.is_ok(), "Failed to create and fill file");

    let meta = std::fs::metadata(format!("{}/1.h2w", p)).unwrap();
    assert_eq!(meta.len(), 512);

    std::fs::remove_file(format!("{}/1.h2w", p)).unwrap();
}
