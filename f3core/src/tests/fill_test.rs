#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::{Read, Seek, SeekFrom};
#[cfg(test)]
use tempfile::tempfile;

#[cfg(test)]
use crate::{
    file_fill::{fill_buffer, write_chunk},
    flow::DynamicBuffer,
};
#[cfg(test)]
const SECTOR_SIZE: usize = 512;

#[test]
fn test_fill_buffer_sector() {
    let mut buf = vec![0u8; SECTOR_SIZE];
    let len: usize = buf.len();
    let off0: u64 = 1234;
    let off1 = fill_buffer(&mut buf[..], len, off0);
    assert_eq!(off1, off0 + SECTOR_SIZE as u64);
    let first = u64::from_ne_bytes(buf[0..8].try_into().unwrap());
    assert_eq!(first, off0);
}

#[test]
fn test_write_chunk_and_read_back() {
    let mut dbuf = DynamicBuffer::new();
    let mut file: File = tempfile().unwrap();
    let mut offset: u64 = 0;
    let sector_size = SECTOR_SIZE;

    // Writing to one sector
    write_chunk(&mut dbuf, &mut file, sector_size, &mut offset).unwrap();
    assert_eq!(offset, SECTOR_SIZE as u64);

    // REad it and compare
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut data = vec![0u8; SECTOR_SIZE];
    file.read_exact(&mut data).unwrap();
    let first = u64::from_ne_bytes(data[0..8].try_into().unwrap());
    assert_eq!(first, 0);
}
