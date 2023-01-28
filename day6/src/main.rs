use color_eyre::Result;
use counter::Counter;
use std::io::{self, Read};

fn start_of_packet(s: &str) -> Option<usize> {
    let mut count: Counter<u8> = Counter::new();
    let s = s.to_string();
    let bytes = s.as_bytes();

    for (i, end) in bytes.iter().enumerate() {
        count[end] += 1;

        if i < 4 {
            continue;
        }

        let start = &bytes[i - 4];
        count[start] -= 1;

        let good_values = count.values().all(|&v| v == 0 || v == 1);
        let four_ones = count.values().filter(|&&v| v == 1).count() == 4;

        if good_values && four_ones {
            return Some(i + 1);
        }
    }

    None
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read input");

    let start = start_of_packet(&input);
    println!("start of packet: {start:?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index() {
        let start = start_of_packet;

        assert_eq!(start("mjqjpqmgbljsphdztnvjfqwrcgsmlb"), Some(7));
        assert_eq!(start("bvwbjplbgvbhsrlpgdmjqwftvncz"), Some(5));
        assert_eq!(start("nppdvjthqldpwncqszvftbrmjlhg"), Some(6));
        assert_eq!(start("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), Some(10));
        assert_eq!(start("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), Some(11));
    }
}
