use crate::*;
use std::mem::size_of;

// ---- helpers ----

#[inline(always)]
fn write_word_ne(buf: &mut [u8], i: usize, val: u64) {
    let start = i * 8;
    buf[start..start + 8].copy_from_slice(&val.to_ne_bytes());
}

fn gen_ok_sector(expected_offset: u64) -> [u8; SECTOR_SIZE] {
    assert_eq!(SECTOR_SIZE % size_of::<u64>(), 0);
    let num_words = SECTOR_SIZE / size_of::<u64>();

    let mut sector = [0u8; SECTOR_SIZE];

    // word[0] = expected_offset
    write_word_ne(&mut sector, 0, expected_offset);

    // words[1..] = random_number(chain)
    let mut rn = expected_offset;
    for i in 1..num_words {
        rn = random_number(rn);
        write_word_ne(&mut sector, i, rn);
    }
    sector
}

/// Mutates N 64-bit words (with indices >= 1) to force a desired class.
/// Does not touch the header word at index 0.
fn bump_words(sector: &mut [u8], word_indices: &[usize]) {
    for &i in word_indices {
        let start = i * 8;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&sector[start..start + 8]);
        let v = u64::from_ne_bytes(arr) + 1;
        sector[start..start + 8].copy_from_slice(&v.to_ne_bytes());
    }
}


fn assert_counts(s: &FileStats, ok: u64, corrupted: u64, changed: u64, overwritten: u64) {
    assert_eq!(s.secs_ok, ok, "secs_ok");
    assert_eq!(s.secs_corrupted, corrupted, "secs_corrupted");
    assert_eq!(s.secs_changed, changed, "secs_changed");
    assert_eq!(s.secs_overwritten, overwritten, "secs_overwritten");
}

// ---- tests for check_sector ----

#[test]
fn sector_ok() {
    let mut stats = FileStats::new();
    let sector = gen_ok_sector(0);
    check_sector(&sector, 0, &mut stats);
    assert_counts(&stats, 1, 0, 0, 0);
}

#[test]
fn sector_changed_le_tolerance() {
    let mut stats = FileStats::new();
    let mut sector = gen_ok_sector(512); // любой корректный offset

    // Повредим ровно TOLERANCE слов, НО не заголовок (индекс 0)
    let mut to_bump = Vec::new();
    for i in 1..=TOLERANCE {
        to_bump.push(i); // 1..=TOLERANCE
    }
    bump_words(&mut sector, &to_bump);

    check_sector(&sector, 512, &mut stats);
    assert_counts(&stats, 0, 0, 1, 0);
}

#[test]
fn sector_corrupted_gt_tolerance_header_ok() {
    let mut stats = FileStats::new();
    let mut sector = gen_ok_sector(1024);

    // Corrupt TOLERANCE+1 words (not chanhging offset)
    let mut to_bump = Vec::new();
    for i in 1..=(TOLERANCE + 1) {
        to_bump.push(i);
    }
    bump_words(&mut sector, &to_bump);

    check_sector(&sector, 1024, &mut stats);
    assert_counts(&stats, 0, 1, 0, 0);
}

#[test]
fn sector_overwritten_header_wrong_errors_le_tolerance() {
    let mut stats: FileStats = FileStats::new();
    let mut sector = gen_ok_sector(0);

    // Getting "overwritten": offset != expected_offset,
    // but the other words match (error_count == 0 <= TOLERANCE)
    write_word_ne(&mut sector, 0, 0);

    check_sector(&sector, 2048, &mut stats);
    assert_counts(&stats, 0, 0, 0, 1);
}

#[test]
fn sector_corrupted_header_wrong_errors_gt_tolerance() {
    let mut stats = FileStats::new();
    let mut sector = gen_ok_sector(4096);

    // Заголовок неверный + > TOLERANCE повреждений внутри
    write_word_ne(&mut sector, 0, 0xDEAD_BEEF);
    let mut to_bump = Vec::new();
    for i in 1..=(TOLERANCE + 1) {
        to_bump.push(i);
    }
    bump_words(&mut sector, &to_bump);

    check_sector(&sector, 4096, &mut stats);
    assert_counts(&stats, 0, 1, 0, 0);
}

// ---- test for check_buffer (some secs in a row) ----

#[test]
fn buffer_three_sectors_all_ok() {
    let mut stats = FileStats::new();
    let expected_offset = 0u64;

    let s0 = gen_ok_sector(0);
    let s1 = gen_ok_sector(512);
    let s2 = gen_ok_sector(1024);

    let mut buf = Vec::with_capacity(SECTOR_SIZE * 3);
    buf.extend_from_slice(&s0);
    buf.extend_from_slice(&s1);
    buf.extend_from_slice(&s2);

    let new_off = check_buffer(&buf, buf.len(), expected_offset, &mut stats);

    assert_eq!(new_off, 1536);
    assert_counts(&stats, 3, 0, 0, 0);
}
