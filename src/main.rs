use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
struct Process {
    cpu_usage: String,
    name: String,
    id: u32,
    cmdline: String,
}

#[derive(Serialize, Deserialize)]
struct Tree {
    process: Process,
    children: Option<Vec<Box<Tree>>>
}


fn stats(pid: u32) -> Process {
    let path = PathBuf::new()
        .join("/proc")
        .join(pid.to_string());

    let cmdline : String = fs::read_to_string(path.join("cmdline")).unwrap_or_default();
    let id : u32 = pid;
    let stats = fs::read_to_string(path.join("stat")).unwrap_or_default();
    let stats : Vec<_>= stats.split(" ").collect();

    // TODO: calc cpu_usage

    let cpu_usage = stats[13].to_string();
    let name = &stats[1][1..&stats[1].len()-1];
    let name = name.to_string();
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

fn ptree_userspace() -> Option<Tree> {
    ptree(1)
}

fn ptree_kernel() -> Option<Tree> {
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

fn main() {
    let pstree_kernelspace = ptree_kernel();
    let pstree_userspace = ptree_userspace();

    println!("Kernelspace:");
    println!("{}",serde_json::to_string(&pstree_kernelspace).expect("error during parsing"));
    println!("Userspace:");
    println!("{}",serde_json::to_string(&pstree_userspace).expect("error during parsing"));

}
