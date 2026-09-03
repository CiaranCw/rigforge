fn main() -> eframe::Result<()> {
    let options = rigforge_workbench::WorkbenchApp::native_options();
    eframe::run_native(
        rigforge_workbench::i18n::WINDOW_TITLE,
        options,
        Box::new(|cc| {
            rigforge_workbench::install_cjk_fonts(&cc.egui_ctx);
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
