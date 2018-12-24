mod collections;
mod error;
mod ffprobe;
mod index;
mod scan;
mod utils;

use std::io;
use std::path::Path;

fn main() {
    // let index = index::Index::load_or_create_index(".index").unwrap();
    // println!("{:#?}", &index.lookup("Logan's run", None)[..3]);
    // loop {
    //     let mut line = String::new();
    //     io::stdin().read_line(&mut line).unwrap();
    //     println!("{:#?}", &index.lookup(&line.trim(), None)[..3]);
    // }
    let mut t = scan::tree::Tree::new();
    let root = t.insert_root("root");
    let child1 = t.insert_below(root, "child1");
    let child1_1 = t.insert_below(child1, "child1-1");
    let child1_2 = t.insert_below(child1, "child1-2");
    let child2 = t.insert_below(root, "child2");
    let child2_1 = t.insert_below(child2, "child2-1");
    let child3 = t.insert_below(root, "child3");

    println!(
        "{:#?}",
        t.descendants(root).map(|n| t.data(n)).collect::<Vec<_>>()
    );
}
