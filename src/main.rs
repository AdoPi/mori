mod pstree;
use eframe::egui;
use egui::ScrollArea;

fn main_egui() {
    let mut native_options = eframe::NativeOptions::default();
    eframe::run_native("Mori PS", native_options, Box::new(|cc| Box::new(MoriTreeApp::new(cc)))).unwrap();
}


#[derive(Default)]
struct MoriTreeApp {}

impl MoriTreeApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

fn ui_build_tree(ui: &mut egui::Ui) {
    let pstree_userspace =  pstree::userspace_tree();
    pstree_userspace.unwrap().ui(ui);
}


impl eframe::App for MoriTreeApp {

   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

       egui::SidePanel::left("Mori process").show(ctx, |ui| {
               ScrollArea::vertical().show(ui, |ui| {
                   ui_build_tree(ui);
               });
       });

       egui::CentralPanel::default().show(ctx, |ui| {
           // TODO history of process
               use egui::plot::{Line, Plot, PlotPoints};
               let sin: PlotPoints = (0..1000).map(|i| {
                   let x = i as f64 * 0.01;
                   [x, x.sin()]
               }).collect();
               let line = Line::new(sin);
               Plot::new("Usage").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));

       });

       // TODO: repaint every x ms, not every frame
       ctx.request_repaint();
   }

}

fn _main_json() {
    let pstree_kernelspace = pstree::kernel_tree();
    let pstree_userspace =  pstree::userspace_tree();
    println!("Kernelspace:");
    println!("{}",serde_json::to_string(&pstree_kernelspace).expect("error during parsing"));
    println!("Userspace:");
    println!("{}",serde_json::to_string(&pstree_userspace).expect("error during parsing"));
}

fn main() {
    main_egui();
}
