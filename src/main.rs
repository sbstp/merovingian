#![allow(dead_code)]

mod flicks;

use std::io;
use std::path::{Path, PathBuf};

use crate::flicks::*;

fn main() {
    // let index = index::Index::load_or_create_index(".index").unwrap();
    // println!("{:#?}", &index.lookup("Logan's run", None)[..3]);
    // loop {
    //     let mut line = String::new();
    //     io::stdin().read_line(&mut line).unwrap();
    //     println!("{:#?}", &index.lookup(&line.trim(), None)[..3]);
    // }

    // let root = vfs::walk("/home/simon/tank/downloads").unwrap();
    // let movies = scan::scan(&root);

    let mut manager = transfer::Manager::new();
    manager.add_transfer("a.txt", "b.txt");

    while manager.has_work() {
        manager.tick();
    }

    println!("{:#?}", manager);
}
