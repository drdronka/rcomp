#![allow(unused)]

use std::fs;
use std::env;
use log::{debug, error, log_enabled, info, Level, LevelFilter};
use env_logger::Builder;
use std::process;

struct Stats {
    byte: [u32; 256],
}

enum NodeType {
    Branch,
    Leaf(u8),
}

struct Node {
    data: NodeType,
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
        for (idx, weight) in stats.byte.iter().enumerate() {
            if *weight > 0 {
                let mut new_node = Box::new( Node { 
                    data: NodeType::Leaf(idx as u8), 
                    weight: *weight,
                    next: None,
                    left: None,
                    right: None});
                treelist.add_sorted(new_node);
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
                        NodeType::Leaf(data) => {
                            debug!("({:02X}:{})", data, node.weight);
                            tmp_node = &node.next;
                        }
                        _ => (),
                    }
                },
                None => return,
            }
        }
    }

//    fn add_sorted(&mut self, byte: u8, weight: u32) {    
    fn add_sorted(&mut self, mut new_node: Box<Node>) {
        match new_node.data {
            NodeType::Branch => debug!("adding branch: {}", new_node.weight),
            NodeType::Leaf(byte) => debug!("adding leaf: ({:02X}:{})", byte, new_node.weight),
        };
        match &self.root {
            None => { 
                // add as only
                debug!("-> head (first)");
                self.root = Some(new_node);
                return;
            },
            Some(node) => {
                // add as first
                if node.weight >= new_node.weight {
                    debug!("-> head");
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
                            debug!("-> tail");
                            node1.next = Some(new_node);
                            return;
                        },
                        Some(node2) => {
                            if node2.weight >= new_node.weight {
                                debug!("-> middle");
                                new_node.next = node1.next.take();
                                node1.next = Some(new_node);
                                return;
                            }
                            else {
                                //debug!("iterating");
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

    fn has_elements(&self, n: u32) -> bool {
        let mut cnt = 0u32;
        let mut tmp_node = &self.root;
        loop {
            match tmp_node {
                Some(node) => {
                    cnt += 1;
                    if cnt >= n {
                        return true;
                    }
                    tmp_node = &node.next;
                },
                None => return false,
            }
        }
    }

    fn transform(&mut self) {
        while self.has_elements(2) {
            let mut new_branch = Box::new(Node {
                data: NodeType::Branch,
                weight: 0,
                next: None,
                left: None,
                right: None
            });
            new_branch.left = self.root.take();
            new_branch.right = match new_branch.left {
                Some(ref mut node) => {
                    new_branch.weight += node.weight;
                    node.next.take()
                },
                None => {
                    error!("invalid state");
                    return;
                }
            };
            self.root = match new_branch.right {
                Some(ref mut node) => { 
                    new_branch.weight += node.weight;
                    node.next.take()
                },
                None => {
                    error!("invalid state");
                    return;
                }
            };
            self.add_sorted(new_branch);
        }
    }
}

fn compress(path_in: &str, path_out: &str) {
    info!("starting huffman compression");
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

    info!("creating treelist");
    let mut treelist = Treelist::from_stats(&stats);
    if log_enabled!(Level::Debug) {
        treelist.print();
    }
    info!("transforming treelist");
    treelist.transform();
}

fn main() {
    Builder::new().filter_level(LevelFilter::Debug).init();

    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        info!("usage: {} [src_file] [dst_file]", args[0].split('/').last().unwrap());
        return;
    }

    compress(args[1].as_str(), args[2].as_str());
}
