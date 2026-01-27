#![allow(unused)]

use std::fs;
use std::env;
use log::{debug, error, log_enabled, info, Level};

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
        debug!("stats:");
        for (idx, elem) in self.byte.iter().enumerate() {
            if *elem > 0 {
                debug!("({:02X}:{})", idx, *elem);
            }
        } 
    }
}

impl Treelist {
    fn from_stats(stats: &Stats) -> Treelist {
        let mut treelist = Treelist { root: None };
        for (idx, elem) in stats.byte.iter().enumerate() {
            if *elem > 0 {
                treelist.add_sorted(idx as u8, *elem as u32);
            }
        }
        treelist
    }

    fn print(&self) {
        debug!("treelist:");
        let mut tmp_node = &self.root;
        loop {
            match &tmp_node {
                Some(node) => {
                    match &node.data {
                        NodeData::Leaf(data) => {
                            debug!("{:02X}: {}", data, node.weight);
                            tmp_node = &node.next;
                        }
                        _ => (),
                    }
                },
                None => return,
            }
        }
    }

    fn add_sorted(&mut self, byte: u8, weight: u32) {    
        let mut new_node = Box::new( Node { 
            data: NodeData::Leaf(byte), 
            weight: weight,
            next: None,
            left: None,
            right: None});

        match &self.root {
            None => { 
                // add as only
                debug!("adding only ({:02X}:{})", byte, weight);
                self.root = Some(new_node);
                return;
            },
            Some(node) => {
                // add as first
                if node.weight >= weight {
                    debug!("adding first ({:02X}:{})", byte, weight);
                    new_node.next = self.root.take();
                    self.root = Some(new_node);
                    return;
                }
            }
        }

        let mut tmp_node = &mut self.root;
        loop {
            match tmp_node {
                Some(node1) => {
                    match &node1.next {
                        None => {
                            debug!("adding last ({:02X}:{})", byte, weight);
                            node1.next = Some(new_node);
                            return;
                        },
                        Some(node2) => {
                            if node2.weight >= weight {
                                debug!("adding middle ({:02X}:{})", byte, weight);
                                new_node.next = node1.next.take();
                                node1.next = Some(new_node);
                                return;
                            }
                            else {
                                debug!("iterating");
                                tmp_node = &mut node1.next;
                            }
                        },
                    }
                },
                _ => {
                    error!("invalid state");
                    return;
                },
            }
        }
    }
}

fn compress(path_in: &str, path_out: &str) {
    info!("starting compression");
    info!("input file: {}", path_in);
    info!("output file: {}", path_out);

    info!("calculating stats");
    let stats: Stats = match Stats::from_file(path_in) {
        Ok(stats) => {
            if log_enabled!(Level::Debug) {
                stats.print();
            }
            stats
        },
        Err(msg) => {
            error!("failed to calculate stats: {}", msg);
            return;
        },
    };

    info!("calculating huffman tree");
    let treelist = Treelist::from_stats(&stats);
    if log_enabled!(Level::Debug) {
        treelist.print();
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        info!("usage: {} [src_file] [dst_file]", args[0].split('/').last().unwrap());
        return;
    }

    compress(args[1].as_str(), args[2].as_str());
}
