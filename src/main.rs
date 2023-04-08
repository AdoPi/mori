use std::path::PathBuf;
use std::fs;

struct Process {
    cpu_usage: String,
    id: u32,
    cmdline: String,
}

struct Tree {
    proc: Process,
    children: Option<Vec<Box<Tree>>>
}


fn stats(pid: u32) -> Process {
    let path = PathBuf::new()
        .join("/proc")
        .join(pid.to_string());

    let cmdline : String = fs::read_to_string(path.join("cmdline")).unwrap_or_default();
    let id : u32 = pid;
    let cpu_usage : String = fs::read_to_string(path.join("stat")).unwrap_or_default();

    Process {
        cpu_usage,
        id,
        cmdline,
    }
}

// TODO: improve clone() and u32 params, could be better if we just pass a String to this function
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

fn ptree_userspace() -> Option<Tree> {
    Some(build_tree(Tree {
        proc: stats(1),
        children: None,
    }))
}

// TODO: remove None in children, but how?
fn ptree_kernel() -> Option<Tree> {
    Some(build_tree(Tree {
        proc: stats(2),
        children: None,
    }))
}

// recursively building a tree
fn build_tree(tree: Tree) -> Tree {
    let pid = tree.proc.id;
    let children = children(pid);

    if children.is_empty() {
        return tree;
    }

    // TODO
    let mut v : Vec<Box<Tree>> = Vec::new();

    for i in children.trim().split(" ") {
        let child_pid : u32 = i.parse().expect("Can't convert id of pid");

        let pchild = stats(child_pid);
        v.push(Box::new(build_tree(Tree {proc: pchild, children: None})));
    }

    return Tree {
        proc: tree.proc,
        children: Some(v)
    };

}


fn main() {
    ptree_kernel();
    ptree_userspace();
}
