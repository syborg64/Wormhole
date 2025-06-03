use crate::pods::pod::TreeLine;

pub fn display_tree(tree: Vec<TreeLine>) -> String {
    let mut output = String::new();
    for (indent, ino, path, hosts) in tree {
        output.push_str(&format!(
            "{}[{ino}] {}    ->    ({}) {:?}\n",
            generate_indentation(indent),
            path,
            hosts.len(),
            hosts
        ));
    }
    output
}

fn generate_indentation(n: u8) -> String {
    let result = " |  ".to_string();
    result.repeat(n.into())
}
