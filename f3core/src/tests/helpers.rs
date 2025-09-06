use crate::{
    utils::{SECTOR_SIZE, random_number},
    verify::FileStats,
};

// ---- helpers ----

#[inline(always)]
pub fn write_word_ne(buf: &mut [u8], i: usize, val: u64) {
    let start = i * 8;
    buf[start..start + 8].copy_from_slice(&val.to_ne_bytes());
}

pub fn gen_ok_sector(expected_offset: u64) -> [u8; SECTOR_SIZE] {
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
pub fn bump_words(sector: &mut [u8], word_indices: &[usize]) {
    for &i in word_indices {
        let start = i * 8;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&sector[start..start + 8]);
        let v = u64::from_ne_bytes(arr) + 1;
        sector[start..start + 8].copy_from_slice(&v.to_ne_bytes());
    }
}

pub fn assert_counts(s: &FileStats, ok: u64, corrupted: u64, changed: u64, overwritten: u64) {
    assert_eq!(s.secs_ok(), ok, "secs_ok");
    assert_eq!(s.secs_corrupted(), corrupted, "secs_corrupted");
    assert_eq!(s.secs_changed(), changed, "secs_changed");
    assert_eq!(s.secs_overwritten(), overwritten, "secs_overwritten");
}
