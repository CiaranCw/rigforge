fn main() -> eframe::Result<()> {
    let options = rigforge_workbench::WorkbenchApp::native_options();
    eframe::run_native(
        "RigForge Workbench",
        options,
        Box::new(|_cc| Ok(Box::new(rigforge_workbench::WorkbenchApp::empty()))),
    )
}
