#![allow(unused)]

use env_logger::Builder;
use log::{debug, error, info, log_enabled, Level, LevelFilter};
use std::env;
use std::fs;
use std::process;

struct Stats {
    weight: [u32; 256],
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

struct CodingTable {
    code: [Vec<u8>; 256],
}

struct BitArray {
    data: Vec<u8>,
    tmp: u8,
    tmp_idx: u8,
}

impl BitArray {
    fn encode(path_in: &str, coding_table: &CodingTable) -> Result<BitArray, std::io::Error> {
      let data = fs::read(path_in)?;
      let mut bit_array = BitArray { data: Vec::<u8>::new(), tmp: 0, tmp_idx: 0 };
      for chr in data {
        bit_array.add_char(&coding_table.code[chr as usize]);
      }
      bit_array.finish();
      Ok(bit_array)
    }
 
    fn add_char(&mut self, code: &Vec<u8>) {
      for bit in code.iter() {
        self.tmp |= bit << self.tmp_idx;
        self.tmp_idx += 1;
        if self.tmp_idx >= 8 {
          debug!("({:08b})", self.tmp);
          self.data.push(self.tmp);
          self.tmp = 0;
          self.tmp_idx = 0;
        }
      }
    }
    fn finish(&mut self) {
      debug!("({:08b}) - remainder", self.tmp);
      self.data.push(self.tmp);
      debug!("({:08b}) - remainder size ({})", self.tmp_idx, self.tmp_idx);
      self.data.push(self.tmp_idx);
    }
}

impl CodingTable {
    fn from_treelist(treelist: &Treelist) -> Result<CodingTable, &str> {
        let mut coding_table = CodingTable {
            code: std::array::from_fn(|_| Vec::new()),
        };
        match &treelist.root {
            Some(node) => {
                coding_table.add_code(node, &mut Vec::<u8>::new());
            }
            None => {
                return Err("invalid treelist");
            }
        }
        Ok(coding_table)
    }
    fn add_code(&mut self, root: &Node, curr_code: &mut Vec<u8>) -> Result<(), &str> {
        match &root.data {
            NodeType::Branch => {
                match &root.left {
                    Some(node_left) => {
                        curr_code.push(0);
                        debug!("traversing <--");
                        self.add_code(node_left, curr_code);
                        curr_code.pop();
                    }
                    None => {
                        return Err("invalid branch");
                    }
                }
                match &root.right {
                    Some(node_right) => {
                        curr_code.push(1);
                        debug!("traversing -->");
                        self.add_code(node_right, curr_code);
                        curr_code.pop();
                    }
                    None => {
                        return Err("invalid branch");
                    }
                }
            }
            NodeType::Leaf(chr) => {
                self.code[*chr as usize] = curr_code.clone();
                debug!(
                    "code: ({:02X}: {})",
                    chr,
                    curr_code
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                );
            }
        }
        Ok(())
    }
    fn print(&self) {
        debug!("coding table:");
        for (chr, code) in self.code.iter().enumerate() {
            if code.len() > 0 {
                debug!(
                    "({:02X}:{})",
                    chr,
                    code.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                );
            }
        }
    }
}

impl Stats {
    fn from_file(path_in: &str) -> Result<Stats, std::io::Error> {
        let data = fs::read(path_in)?;
        Ok(Stats::from_data(&data))
    }

    fn from_data(data: &Vec<u8>) -> Stats {
        let mut stats = Stats { weight: [0; 256] };
        for chr in data.iter() {
            stats.weight[*chr as usize] += 1;
        }
        stats
    }

    fn print(&self) {
        debug!("stats:");
        for (chr, weight) in self.weight.iter().enumerate() {
            if *weight > 0 {
                debug!("({:02X}:{})", chr, *weight);
            }
        }
    }
}

impl Treelist {
    fn from_stats(stats: &Stats) -> Treelist {
        let mut treelist = Treelist { root: None };
        for (idx, weight) in stats.weight.iter().enumerate() {
            if *weight > 0 {
                let mut new_node = Box::new(Node {
                    data: NodeType::Leaf(idx as u8),
                    weight: *weight,
                    next: None,
                    left: None,
                    right: None,
                });
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
                            debug!("leaf ({:02X}:{})", data, node.weight);
                        }
                        NodeType::Branch => {
                            debug!("branch ({})", node.weight);
                        }
                    }
                    tmp_node = &node.next;
                }
                None => return,
            }
        }
    }

    fn add_sorted(&mut self, mut new_node: Box<Node>) {
        match new_node.data {
            NodeType::Branch => {
                debug!("adding branch: {}", new_node.weight);
            }
            NodeType::Leaf(chr) => {
                debug!("adding leaf: ({:02X}:{})", chr, new_node.weight);
            }
        }
        match &self.root {
            None => {
                // add as only
                debug!("-> head (first)");
                self.root = Some(new_node);
                return;
            }
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
                        }
                        Some(node2) => {
                            if node2.weight >= new_node.weight {
                                debug!("-> middle");
                                new_node.next = node1.next.take();
                                node1.next = Some(new_node);
                                return;
                            } else {
                                //debug!("iterating");
                                tmp_node = &mut node1.next;
                            }
                        }
                    }
                }
                _ => {
                    error!("invalid state");
                    return;
                }
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
                }
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
                right: None,
            });
            new_branch.left = self.root.take();
            new_branch.right = match new_branch.left {
                Some(ref mut node) => {
                    new_branch.weight += node.weight;
                    node.next.take()
                }
                None => {
                    error!("invalid state");
                    return;
                }
            };
            self.root = match new_branch.right {
                Some(ref mut node) => {
                    new_branch.weight += node.weight;
                    node.next.take()
                }
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
    let stats = match Stats::from_file(path_in) {
        Ok(stats) => {
            if log_enabled!(Level::Debug) {
                stats.print();
            }
            stats
        }
        Err(msg) => {
            error!("failed to calculate stats: {}", msg);
            return;
        }
    };

    info!("creating treelist");
    let mut treelist = Treelist::from_stats(&stats);
    if log_enabled!(Level::Debug) {
        treelist.print();
    }

    info!("transforming treelist");
    treelist.transform();
    if log_enabled!(Level::Debug) {
        treelist.print();
    }

    let coding_table = match CodingTable::from_treelist(&treelist) {
        Ok(table) => {
            if log_enabled!(Level::Debug) {
                table.print();
            }
            table
        }
        Err(msg) => {
            error!("{}", msg);
            return;
        }
    };

    info!("encoding data");
    let bit_array = BitArray::encode(path_in, &coding_table); 
}

fn main() {
    Builder::new().filter_level(LevelFilter::Debug).init();

    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        info!(
            "usage: {} [src_file] [dst_file]",
            args[0].split('/').last().unwrap()
        );
        return;
    }

    compress(args[1].as_str(), args[2].as_str());
}
