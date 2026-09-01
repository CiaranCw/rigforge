use rigforge_app::Application;
use rigforge_workbench::{PreviewEmbeddingSlot, WorkbenchApp};

#[test]
fn shell_initializes_without_network_or_blender() {
    assert!(!WorkbenchApp::requires_network());
    assert!(!WorkbenchApp::requires_blender());
    assert!(!Application::requires_network());
    assert!(!Application::requires_blender());
    let _options = WorkbenchApp::native_options();
    let slot = PreviewEmbeddingSlot::default();
    assert!(!slot.occupied);
    assert_eq!(PreviewEmbeddingSlot::viewer_library(), None);
    assert_eq!(PreviewEmbeddingSlot::payload_format(), None);
}

#[test]
fn workbench_reads_application_lists_without_blender() {
    let app = Application::open_in_memory().unwrap();
    let shell = WorkbenchApp::from_application(&app).unwrap();
    assert!(shell.selected_character_version().is_none());
    assert!(shell.job_states().is_empty());
}
