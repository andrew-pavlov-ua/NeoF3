// f3-write/tests/write_tests.rs

use crate::*;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use tempfile::tempfile;

use f3core::flow::{DynamicBuffer, Flow};

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

#[test]
fn integration_create_one_sector() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().to_str().unwrap();

    let mut flow = Flow::new(512, -1, false);
    let stop = create_and_fill_file(p, 1, 512, false, &mut flow);
    assert!(stop.is_ok(), "Failed to create and fill file");

    let meta = std::fs::metadata(format!("{}/1.h2w", p)).unwrap();
    assert_eq!(meta.len(), 512);

    std::fs::remove_file(format!("{}/1.h2w", p)).unwrap();
}
