use anyhow::Result;
use dom::html;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

fn main() -> Result<()> {
    // domのバイナリを作る
    let mut html_file = OpenOptions::new()
        .read(true)
        .open("./sample/sample.html")
        .unwrap();
    let mut html = String::new();
    html_file.read_to_string(&mut html).unwrap();
    let node = html::parse(html.as_str());
    let mut bin_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("./sample/sample.html.bin")
        .unwrap();
    let bin = bincode::serialize(&node)?;
    bin_file.write_all(&bin)?;
    Ok(())
}
