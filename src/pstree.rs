use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Process {
    pub cpu_usage: String,
    pub name: String,
    pub id: u32,
    pub cmdline: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tree {
    pub process: Process,
    pub children: Option<Vec<Box<Tree>>>
}

impl Default for Tree {
    fn default() -> Self {
        Tree {
            process: Process {
                cpu_usage: String::new(),
                name: String::new(),
                id: 0,
                cmdline: String::new(),
            },
            children: None
        }
    }
}


fn stats(pid: u32) -> Process {
    let path = PathBuf::new()
        .join("/proc")
        .join(pid.to_string());

    let cmdline : String = fs::read_to_string(path.join("cmdline")).unwrap_or_default();
    let id : u32 = pid;
    let stats = fs::read_to_string(path.join("stat")).unwrap_or_default();
    let stats : Vec<_>= stats.split(" ").collect();

    if 13 > stats.len() {
        return Process {
            cpu_usage: "0".to_string(),
            name: "".to_string(),
            id: pid,
            cmdline
        }
    }

    let mut cpu_usage = stats[13].to_string();
    let name = &stats[1][1..&stats[1].len()-1];
    let name = name.to_string();

    // Calc cpu_usage
    if stats.len() >= 20 {
        let utime : i64 = stats[13].parse().unwrap_or(0);
        let stime : i64 = stats[14].parse().unwrap_or(0);
        let cutime : i64 = stats[15].parse().unwrap_or(0);
        let cstime : i64 = stats[16].parse().unwrap_or(0);
        let starttime : i64 = stats[21].parse().unwrap_or(0);

        let total_time : i64 = utime + stime;
        let total_time : i64 = total_time + cutime  + cstime; // include children time

        let path = PathBuf::new()
            .join("/proc");
        let uptime = fs::read_to_string(path.join("uptime")).unwrap_or_default();
        let uptime = uptime.split(".").next(); // TODO: LOCALE needs to be EN
        let uptime = uptime.unwrap();
        let uptime : i64 = uptime.parse().unwrap();

        fn get_hertz() -> i64 {
            unsafe { libc::sysconf(libc::_SC_CLK_TCK) }
        }

        let seconds :f64 = uptime as f64 - (starttime / get_hertz()) as f64;
        let usage : f64 = 100.0 * ( (total_time / get_hertz())) as f64 / seconds;
        cpu_usage = usage.to_string();

    }

    Process {
        cpu_usage,
        name,
        id,
        cmdline,
    }
}

fn children(pid: u32) -> String {
    let pid = pid.to_string();
    let path = PathBuf::new()
        .join("/proc")
        .join(pid.clone())
        .join("task")
        .join(pid.clone())
        .join("children");

    fs::read_to_string(path).unwrap_or_default()
}

fn ptree(root_pid: u32 ) -> Option<Tree> {
    Some(build_tree(Tree {
        process: stats(root_pid),
        children: None,
    }))
}


fn ptree_with_stats(root_pid: u32, map: &mut std::collections::HashMap<u32,Vec<f64>>) -> Option<Tree> {
    Some(build_tree_with_stats(Tree {
        process: stats(root_pid),
        children: None,
    }, map))
}

pub fn userspace_tree() -> Option<Tree> {
    ptree(1)
}


pub fn userspace_tree_with_stats(map: &mut std::collections::HashMap<u32,Vec<f64>>) -> Option<Tree> {
    ptree_with_stats(1,map)
}

pub fn kernel_tree() -> Option<Tree> {
    ptree(2)
}

// recursively building a tree
fn build_tree(tree: Tree) -> Tree {
    let pid = tree.process.id;
    let children = children(pid);

    if children.is_empty() {
        return tree;
    }

    let mut v : Vec<Box<Tree>> = Vec::new();

    for i in children.trim().split(" ") {
        let child_pid : u32 = i.parse().expect("Can't convert id of pid");

        let pchild = stats(child_pid);
        v.push(Box::new(build_tree(Tree {process: pchild, children: None})));
    }

    return Tree {
        process: tree.process,
        children: Some(v)
    };
}


fn build_tree_with_stats( tree: Tree, map: &mut std::collections::HashMap<u32,Vec<f64>>) -> Tree {

    let pid = tree.process.id;
    let children = children(pid);

    if children.is_empty() {
        return tree;
    }

    let mut v : Vec<Box<Tree>> = Vec::new();

    for i in children.trim().split(" ") {
        let child_pid : u32 = i.parse().expect("Can't convert id of pid");

        let pchild = stats(child_pid);
        // records stats
        let mut vc = map.get_mut(&pchild.id);
        if vc.is_none() {
            map.insert(pchild.id, Vec::new());
        } else {
            let cpu_u : f64 = pchild.cpu_usage.trim().parse().unwrap();
            vc.unwrap().push(cpu_u);
            // insert into map again?
        }

        v.push(Box::new(build_tree_with_stats(Tree {process: pchild, children: None}, map)));
    }

    return Tree {
        process: tree.process,
        children: Some(v)
    };
}



// TODO: move Ui implementation in another file
use egui::Ui;

impl Tree {
    pub fn build() -> Self {
        userspace_tree().unwrap()
    }

    pub fn ui(&mut self, ui: &mut Ui, current_index: &mut u32) {
        let n = self.process.name.clone();
        let n = n.as_str();
        // self.ui_impl(ui, 1, format!("#1 [{}]",n).as_str(),current_index);
        self.ui_impl(ui, 1, n,current_index);
    }

    fn ui_impl(&mut self, ui: &mut Ui, depth: usize, name: &str, current_index: &mut u32) {

        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            name.to_string().clone().into(),
            depth < 1,
        );
        state
            .show_header(ui, |ui| {
                let response = ui.vertical(|ui| {
                    ui.label(name);
                    ui.separator();
                });
                let id = ui.make_persistent_id(name);
                if ui
                    .interact(response.response.rect, id, egui::Sense::click())
                    .clicked()
                {
                    // record pid
                    let mut s = name.split(' ');
                    let s = s.next().unwrap().trim();
                    let s = &s[1..s.len()];
                    if let Ok(c)  = s.parse::<u32>() {
                        *current_index = c;
                    }
                }
            })
            .body(|ui| self.children_ui(ui, depth, current_index));
    }

    fn children_ui(&mut self, ui: &mut Ui, depth: usize, current_index: &mut u32) {
        // WHY .as_mut() ?
        if let Some(children) = self.children.as_mut() {
            for i in children {
                i.ui_impl(ui, depth + 1, &format!("#{} [{}]", i.process.id, i.process.name), current_index);
            }
        }
    }
}
