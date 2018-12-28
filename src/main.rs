#![allow(dead_code)]

mod flicks;

use std::io;
use std::path::{Path, PathBuf};

use crate::flicks::*;

fn main() {
    let index = index::Index::load_or_create_index(".index").unwrap();
    let root = vfs::walk("/home/simon/tank/downloads").unwrap();
    let movies = scan::scan(&root, &index);

    let mut manager = transfer::Manager::new();
    manager.add_transfer("a.txt", "b.txt");

    while manager.has_work() {
        manager.tick();
    }

    println!("{:#?}", manager);
}
