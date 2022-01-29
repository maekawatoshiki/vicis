pub mod module;

use anyhow::Result;
use std::io::{BufRead, Cursor, Read};

pub fn parse<T>(cursor: &mut Cursor<T>) -> Result<()>
where
    T: BufRead + Read + PartialEq + AsRef<[u8]>,
{
    if !parse_magic(cursor, b"\x42\x43\xc0\xde")? {
        return Err(anyhow::anyhow!("Invalid magic number"));
    }
    Ok(())
}

pub fn parse_magic<T>(cursor: &mut Cursor<T>, magic: &[u8]) -> Result<bool>
where
    T: BufRead + Read + PartialEq + AsRef<[u8]>,
{
    let mut buf = vec![0; magic.len()];
    cursor.read_exact(&mut buf)?;
    Ok(buf == magic)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let mut cursor = Cursor::new(b"\x42\x43\xc0\xde".as_slice());
        parse(&mut cursor).unwrap();
    }
}
