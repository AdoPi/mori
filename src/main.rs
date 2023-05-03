mod pstree;
mod treemap;
use eframe::egui;
use egui::ScrollArea;

fn main_egui() {
    let mut native_options = eframe::NativeOptions::default();
    eframe::run_native("Mori PS", native_options, Box::new(|cc| Box::new(MoriTreeApp::new(cc)))).unwrap();
}


#[derive(Default)]
struct MoriTreeApp {
    map: std::collections::HashMap<u32,Vec<f64>>,
    current_pid: u32,
}

impl MoriTreeApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

fn ui_build_tree(ui: &mut egui::Ui, map: Option<&mut std::collections::HashMap<u32,Vec<f64>>>, current_pid: &mut u32) {
    if map.is_none() {
        return;
    }
    let pstree_userspace =  pstree::userspace_tree_with_stats(map.unwrap());
    pstree_userspace.unwrap().ui(ui,current_pid);
}


impl eframe::App for MoriTreeApp {

   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

       egui::SidePanel::left("Mori process").show(ctx, |ui| {
               ScrollArea::vertical().show(ui, |ui| {
                   ui_build_tree(ui,Some(&mut self.map),&mut self.current_pid);
               });
       });

       egui::CentralPanel::default().show(ctx, |ui| {
           ui.label(format!("Process {}",self.current_pid));
           let v = self.map.get(&self.current_pid);
           if v.is_none() {
           } else {
               let s = v.unwrap().iter().map(|e| e.to_string());
               let s : Vec<_> = s.collect(); // TODO: collect::Vec<_>() oneliner

               use egui::plot::{Line, Plot, PlotPoints};
               let g: PlotPoints = (0..s.len()).map(|i| {
                   let mut y : f64 = 0.0;
                   if i >= s.len() {
                       y = 0.0;
                   } else {
                       y = s[i].parse().unwrap_or(0.0);
                   }
                   [i as f64,y as f64]
               }).collect();
               let line = Line::new(g);
               Plot::new("Usage").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));
           }

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
