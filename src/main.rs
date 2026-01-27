#![allow(unused)]

use std::fs;
use std::env;
//use log::debug;

const VERBOSE_EN: bool = true;

struct Stats {
    byte: [u32; 256],
}

enum NodeData {
    Branch,
    Leaf(u8),
}

struct Node {
    data: NodeData,
    weight: u32,
    next: Option<Box<Node>>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

struct Treelist {
    root: Option<Box<Node>>,
}

impl Stats {
    fn from_file(path_in: &str) -> Result<Stats, std::io::Error> {
        let data = fs::read(path_in)?;
        Ok(Stats::from_data(&data))
    }

    fn from_data(data: &Vec<u8>) -> Stats {
        let mut stats = Stats { byte: [0; 256] };
        for val in data.iter() {
            stats.byte[*val as usize] += 1;
        }
        stats
    }

    fn print(&self) {
        println!("stats:");
        for (idx, elem) in self.byte.iter().enumerate() {
            if *elem > 0 {
                println!("{:02X}: {}", idx, *elem);
            }
        } 
    }
}

impl Treelist {
    fn from_stats(stats: &Stats) -> Treelist {
        let mut treelist = Treelist { root: None };
        for (idx, elem) in stats.byte.iter().enumerate() {
            treelist.add(idx as u8, *elem as u32);
        }
        Treelist { root: None }
    }
    fn add(&mut self, byte: u8, weight: u32) {
        let new_node = Box::new( Node { 
            data: NodeData::Leaf(byte), 
            weight: weight,
            next: self.root.take(),
            left: None,
            right: None});
        self.root = Some(new_node);
    }
}

fn compress(path_in: &str, path_out: &str, verbose: bool) {
    println!("starting compression");
    println!("input file: {}", path_in);
    println!("output file: {}", path_out);

    print!("calculating stats.. ");
    let stats: Stats = match Stats::from_file(path_in) {
        Ok(stats) => {
            println!("done");
            if verbose {
                stats.print();
            }
            stats
        },
        Err(msg) => {
            println!("failed: {}", msg);
            return;
        },
    };

    println!("calculating huffman tree");
    let treelist = Treelist::from_stats(&stats);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        println!("usage: {} [src_file] [dst_file]", args[0].split('/').last().unwrap());
        return;
    }

    compress(args[1].as_str(), args[2].as_str(), VERBOSE_EN);

//    let mut asdf = vec![0, 1, 2, 3, 4];
//    asdf.remove(2);    
//    println!("{:?}", asdf);
}
