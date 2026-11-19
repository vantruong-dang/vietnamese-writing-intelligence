//! Bảng Syllable Bigram nén (`.vbg`) — tra cứu zero-copy bằng binary search.
//!
//! Layout:
//! ```text
//! [0..4]   magic "VBG1"
//! [4..6]   version u16 = 1
//! [6..8]   flags u16
//! [8..12]  count u32
//! [12..16] reserved u32
//! [16..16+8N] hashes [u64; N] LE, sorted
//! [16+8N..16+9N] scores [u8; N]
//! ```
//!
//! Hash: FNV-1a 64-bit trên `w1` + `\0` + `w2` (UTF-8, NFC, lowercase).

pub const VBG_MAGIC: &[u8; 4] = b"VBG1";
pub const VBG_VERSION: u16 = 1;

#[inline]
pub fn fnv1a_64_bigram(w1: &str, w2: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in w1.as_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash ^= 0;
    hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    for &byte in w2.as_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Debug, Clone)]
pub struct BigramTable<'a> {
    hashes_bytes: &'a [u8],
    scores: &'a [u8],
    count: usize,
}

impl<'a> BigramTable<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
        if data.len() < 16 || &data[0..4] != VBG_MAGIC {
            return None;
        }
        let version = u16::from_le_bytes(data[4..6].try_into().ok()?);
        if version != VBG_VERSION {
            return None;
        }
        let count = u32::from_le_bytes(data[8..12].try_into().ok()?) as usize;
        let hashes_end = 16usize.checked_add(count.checked_mul(8)?)?;
        let scores_end = hashes_end.checked_add(count)?;
        if data.len() < scores_end {
            return None;
        }
        Some(Self {
            hashes_bytes: &data[16..hashes_end],
            scores: &data[hashes_end..scores_end],
            count,
        })
    }

    #[inline]
    fn hash_at(&self, index: usize) -> u64 {
        let offset = index * 8;
        u64::from_le_bytes(
            self.hashes_bytes[offset..offset + 8]
                .try_into()
                .expect("hash slice"),
        )
    }

    /// 0 = OOV, 1..=255 = log-frequency score.
    pub fn score(&self, w1: &str, w2: &str) -> u8 {
        let target = fnv1a_64_bigram(w1, w2);
        let mut low = 0usize;
        let mut high = self.count;
        while low < high {
            let middle = low + (high - low) / 2;
            let hash = self.hash_at(middle);
            if hash < target {
                low = middle + 1;
            } else if hash > target {
                high = middle;
            } else {
                return self.scores[middle];
            }
        }
        0
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv_hash_known_vectors() {
        // Giá trị khóa cứng từ build_bigram_vbg.py --verify.
        assert_eq!(fnv1a_64_bigram("sáp", "nhập"), 0x2f96_96f1_a4f1_1f06);
        assert_eq!(fnv1a_64_bigram("sát", "nhập"), 0x67f3_f766_31e9_87c2);
        assert_ne!(
            fnv1a_64_bigram("giành", "giật"),
            fnv1a_64_bigram("dành", "giật")
        );
        assert_ne!(fnv1a_64_bigram("a", "b"), fnv1a_64_bigram("ab", ""));
    }

    #[test]
    fn test_table_from_committed_fixture() {
        let bytes = include_bytes!("../../../neural/fixtures/vi_bigram.vbg");
        let table = BigramTable::from_bytes(bytes).expect("parse vbg");
        assert!(table.len() > 1000);
        assert!(table.score("chính", "tả") > 0);
    }

    #[test]
    fn test_rejects_bad_magic_and_short() {
        assert!(BigramTable::from_bytes(b"XXXX").is_none());
        let mut bytes = include_bytes!("../../../neural/fixtures/vi_bigram.vbg").to_vec();
        bytes[0] = b'X';
        assert!(BigramTable::from_bytes(&bytes).is_none());
    }
}
