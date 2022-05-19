use std::{collections::VecDeque, str};

pub fn unescape(s: &str) -> Option<String> {
    let mut queue: VecDeque<_> = String::from(s).chars().collect();
    let mut s = String::new();

    while let Some(c) = queue.pop_front() {
        if c != '\\' {
            s.push(c);
            continue;
        }

        match queue.pop_front() {
            Some('\\') => s.push('\\'),
            Some(c) if c.is_ascii_hexdigit() => {
                let cc = queue.pop_front().unwrap();
                s.push(
                    char::from_u32(
                        u32::from_str_radix(format!("{}{}", c, cc).as_str(), 16).unwrap(),
                    )
                    .unwrap(),
                );
            }
            _ => return None,
        };
    }

    Some(s)
}

pub fn escape(s: &str) -> String {
    return str::from_utf8(
        &s.as_bytes()
            .iter()
            .flat_map(|&c| {
                if c.is_ascii_control() {
                    vec![b'\\', hexify(c >> 4), hexify(c & 0xf)]
                } else {
                    vec![c]
                }
                .into_iter()
            })
            .collect::<Vec<u8>>(),
    )
    .unwrap()
    .to_string();

    fn hexify(b: u8) -> u8 {
        match b {
            0..=9 => b'0' + b,
            _ => b'a' + b - 10,
        }
    }
}
