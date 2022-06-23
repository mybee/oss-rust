use super::errors::Error;
use reqwest::header::{HeaderMap, HeaderName};
use std::collections::HashMap;
use std::vec;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

#[inline]
pub async fn load_file(f: &mut File) -> Result<Vec<u8>, Error> {
    let mut f = BufReader::new(f);
    let mut s = Vec::new();
    f.read_to_end(&mut s).await?;
    Ok(s)
}

#[inline]
pub async fn load_chunk_file(f: &mut File, offset: u64, size: u64) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::with_capacity(size as usize);
    f.seek(SeekFrom::Start(offset)).await?;
    f.take(size).read_to_end(&mut buf).await?;
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
    pub size: u64,
}

// split_file_by_part_size splits big file into parts by the size of parts.
// Splits the file by the part size. Returns the FileChunk when error is nil.
pub async fn split_file_by_part_size(f: &File, chunk_size: u64) -> Result<Vec<FileChunk>, Error> {
    if chunk_size <= 0 {
        return Err(Error::E("chunk_size invalid".to_string()));
    }

    let size = f.metadata().await?.len();

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
            size: chunk_size,
        };
        chunks.push(chunk);
        i = i + 1;
    }

    if size % chunk_size > 0 {
        let chunk = FileChunk {
            number: chunks.len() as u64 + 1,
            offset: chunks.len() as u64 * chunk_size,
            size: size % chunk_size,
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
        let f = tokio::fs::File::open("/tmp/tmp.txt").await.unwrap();
        let res = split_file_by_part_size(&f, 1024).await;
        // println!("res: {:?}", res.unwrap());
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_load_chunk_file() {
        let mut f = tokio::fs::File::open("/tmp/tmp.txt").await.unwrap();
        let data = load_chunk_file(&mut f, 0, 100).await.unwrap();
        println!("data: {:?}", data);
    }
}
