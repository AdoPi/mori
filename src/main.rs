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
    dark_mode: bool,
}

impl MoriTreeApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let ctx = cc.egui_ctx.clone();
        ctx.set_visuals(egui::Visuals::light()); // default to light mode
        std::thread::Builder::new().name("Worker_thread".to_string()).spawn(move || {
            loop {
                // refresh data every X ms
                ctx.request_repaint();
                let d = std::time::Duration::from_millis(100);
                std::thread::sleep(d);
            }
        });

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
           let v = self.map.get(&self.current_pid);
           if v.is_none() {
           } else {
               let s = v.unwrap().iter().map(|e| e.to_string());
               let s : Vec<_> = s.collect();

               use egui::plot::{Line, Plot, PlotPoints};
               let g: PlotPoints = (0..s.len()).map(|i| {
                   let mut y : f64 = 0.0;
                   if let Ok(v) = s[i].parse() {
                           y = v;
                   }
                   // TODO: remove magic numbers
                   [(i as f64/25.0),y as f64]
               }).collect();
               let line = Line::new(g);
               Plot::new("CPU usage").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));
           }

       });

       egui::TopBottomPanel::top("").show(ctx, |ui| {
           ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
               if ui.add(egui::Button::new("Toggle Dark mode")).clicked() {
                   // ctx: egui::Context
                   if self.dark_mode  {
                       ctx.set_visuals(egui::Visuals::light());
                       self.dark_mode = false;
                   }
                   else {
                       ctx.set_visuals(egui::Visuals::dark());
                       self.dark_mode = true;
                   }
               }

               ui.label(format!("Process {}",self.current_pid));
           });
       });

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
