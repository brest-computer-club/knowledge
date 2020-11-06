use std::fs::DirEntry;

pub fn visit(d: &DirEntry) {
    println!("{:?}", d)
}
