#![allow(dead_code)]

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
    let root = scan::vfs::walk("src").unwrap();
    for item in root.descendants() {
        println!("{}", item.path().display());
    }
}
