use color_eyre::Result;
use counter::Counter;
use std::io::{self, Read};

fn unique_window_start(s: &str, window: usize) -> Option<usize> {
    let mut count: Counter<u8> = Counter::new();
    let s = s.to_string();
    let bytes = s.as_bytes();

    for (i, end) in bytes.iter().enumerate() {
        count[end] += 1;

        if i < window {
            continue;
        }

        let start = &bytes[i - window];
        count[start] -= 1;

        let good_values = count.values().all(|&v| v == 0 || v == 1);
        let four_ones = count.values().filter(|&&v| v == 1).count() == window;

        if good_values && four_ones {
            return Some(i + 1);
        }
    }

    None
}

fn packet_start(s: &str) -> Option<usize> {
    unique_window_start(s, 4)
}

fn message_start(s: &str) -> Option<usize> {
    unique_window_start(s, 14)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read input");

    println!("start of packet: {:?}", packet_start(&input));
    println!("start of message: {:?}", message_start(&input));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet() {
        let f = packet_start;
        assert_eq!(f("mjqjpqmgbljsphdztnvjfqwrcgsmlb"), Some(7));
        assert_eq!(f("bvwbjplbgvbhsrlpgdmjqwftvncz"), Some(5));
        assert_eq!(f("nppdvjthqldpwncqszvftbrmjlhg"), Some(6));
        assert_eq!(f("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), Some(10));
        assert_eq!(f("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), Some(11));
    }

    #[test]
    fn message() {
        let f = message_start;
        assert_eq!(f("mjqjpqmgbljsphdztnvjfqwrcgsmlb"), Some(19));
        assert_eq!(f("bvwbjplbgvbhsrlpgdmjqwftvncz"), Some(23));
        assert_eq!(f("nppdvjthqldpwncqszvftbrmjlhg"), Some(23));
        assert_eq!(f("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), Some(29));
        assert_eq!(f("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), Some(26));
    }
}
