use std::fs;

struct Stats {
    byte: [u8; 256],
}
struct Huff {
    stats: Option<Stats>,
}

impl Huff {
    fn calc_stats_from_file(&mut self, path_in: &str) {
        println!("reading file: {path_in}");
        let data = match fs::read(path_in) {
            Ok(data) => {
                println!("read ok");
                data
            }
            Err(_) => {
                println!("read failed");
                return;
            }
        };
        println!("{:?}", data);
        self.stats = Some(self.calc_stats_from_data(&data));
    }

    fn calc_stats_from_data(&self, data: &Vec<u8>) -> Stats {
        let mut stats = Stats { byte: [0; 256] };
        for val in data.iter() {
            stats.byte[*val as usize] += 1;
        }
        println!("data stats");
        for (idx, elem) in stats.byte.iter().enumerate() {
            if *elem > 0 {
                println!("{:#2X}: {}", idx, *elem);
            }
        }
        stats
    }
}

fn main() {
    let path_test_in = "test_in";
    let mut huff = Huff { stats: None };
    huff.calc_stats_from_file(path_test_in);
}
