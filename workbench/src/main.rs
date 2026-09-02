fn main() -> eframe::Result<()> {
    let options = rigforge_workbench::WorkbenchApp::native_options();
    eframe::run_native(
        "RigForge Workbench",
        options,
        Box::new(|_cc| {
            match rigforge_workbench::WorkbenchHost::open_default() {
                Ok(host) => Ok(Box::new(host) as Box<dyn eframe::App>),
                Err(err) => {
                    eprintln!("Workbench Application open failed: {err}");
                    Ok(Box::new(rigforge_workbench::WorkbenchApp::empty()) as Box<dyn eframe::App>)
                }
            }
        }),
    )
}
