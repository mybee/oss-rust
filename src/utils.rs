use super::errors::Error;
use reqwest::header::{HeaderMap, HeaderName};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::os::unix::prelude::FileExt;
use std::vec;

#[inline]
pub fn load_file<S>(p: S) -> Result<Vec<u8>, Error>
where
    S: AsRef<str>,
{
    let p = p.as_ref();
    let f = File::open(p)?;
    let mut f = BufReader::new(f);
    let mut s = Vec::new();
    f.read_to_end(&mut s)?;
    Ok(s)
}

#[inline]
pub fn load_chunk_file<S>(p: S, offset: u64, size: usize) -> Result<Vec<u8>, Error>
where
    S: AsRef<str>,
{
    let p = p.as_ref();
    let f = File::open(p)?;
    let mut buf = vec![0u8; size];
    f.read_at(&mut buf, offset)?;
    Ok(buf)
}

pub fn to_headers<S>(hashmap: HashMap<S, S>) -> Result<HeaderMap, Error>
where
    S: AsRef<str>,
{
    let mut headers = HeaderMap::new();
    for (key, val) in hashmap.iter() {
        let key = key.as_ref();
        let val = val.as_ref();
        headers.insert(HeaderName::from_bytes(key.as_bytes())?, val.parse()?);
    }
    Ok(headers)
}

#[derive(Debug, Clone)]
pub struct FileChunk {
    pub number: u64,
    pub offset: u64,
    pub size: usize,
}

// split_file_by_part_size splits big file into parts by the size of parts.
// Splits the file by the part size. Returns the FileChunk when error is nil.
pub async fn split_file_by_part_size(
    file_name: &str,
    chunk_size: u64,
) -> Result<Vec<FileChunk>, Error> {
    if chunk_size <= 0 {
        return Err(Error::E("chunk_size invalid".to_string()));
    }

    let file = tokio::fs::File::open(file_name).await?;

    let size = file.metadata().await?.len();

    let chunk_n = size / chunk_size;
    if chunk_n >= 10000 {
        return Err(Error::E(
            "Too many parts, please increase part size".to_string(),
        ));
    }

    let mut chunks = vec![];
    let mut i = 0;
    while i < chunk_n {
        let chunk = FileChunk {
            number: i + 1,
            offset: i * chunk_size,
            size: chunk_size as usize,
        };
        chunks.push(chunk);
        i = i + 1;
    }

    if size % chunk_size > 0 {
        let chunk = FileChunk {
            number: chunks.len() as u64 + 1,
            offset: chunks.len() as u64 * chunk_size,
            size: (size % chunk_size) as usize,
        };
        chunks.push(chunk);
    }

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chunk_file() {
        let res = split_file_by_part_size("/tmp/tmp.txt", 1024).await;
        // println!("res: {:?}", res.unwrap());
        assert!(res.is_ok());
    }

    #[test]
    fn test_load_chunk_file() {
        let data = load_chunk_file("/tmp/tmp.txt", 0, 100).unwrap();
        println!("data: {:?}", data);
    }
}
