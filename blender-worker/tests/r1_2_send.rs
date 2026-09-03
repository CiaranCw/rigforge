use rigforge_app::{DispatchReceipt, TerminalOutcome};
use rigforge_blender_worker::BlenderWorker;

fn assert_send<T: Send>() {}

#[test]
fn blender_worker_is_send() {
    assert_send::<BlenderWorker>();
    assert_send::<DispatchReceipt>();
    assert_send::<TerminalOutcome>();
}
