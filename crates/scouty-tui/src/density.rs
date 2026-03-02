//! Braille density chart for the status bar.
//!
//! Uses Unicode Braille characters (U+2800..U+28FF) to render a compact
//! time-density histogram. Each Braille character encodes a 2x4 dot matrix,
//! allowing 2 data points per character with 4 height levels each.
//!
//! Braille dot numbering:
//! ```text
//! 1 4
//! 2 5
//! 3 6
//! 7 8
//! ```
//!
//! We use the left column (dots 1,2,3,7) for one data point and the right
//! column (dots 4,5,6,8) for the next. Height 0 = no dots, height 4 = all 4 dots.

use chrono::{DateTime, Utc};

/// Braille base character (U+2800, empty braille).
const BRAILLE_BASE: u32 = 0x2800;

/// Left column dot bits (bottom to top): dot7=0x40, dot3=0x04, dot2=0x02, dot1=0x01
const LEFT_DOTS: [u32; 4] = [0x40, 0x04, 0x02, 0x01]; // height 1,2,3,4 from bottom

/// Right column dot bits (bottom to top): dot8=0x80, dot6=0x20, dot5=0x10, dot4=0x08
const RIGHT_DOTS: [u32; 4] = [0x80, 0x20, 0x10, 0x08];

/// Build a braille character from left height (0-4) and right height (0-4).
fn braille_char(left_height: usize, right_height: usize) -> char {
    let mut code = BRAILLE_BASE;
    for &dot in LEFT_DOTS.iter().take(left_height.min(4)) {
        code |= dot;
    }
    for &dot in RIGHT_DOTS.iter().take(right_height.min(4)) {
        code |= dot;
    }
    char::from_u32(code).unwrap_or(' ')
}

/// Compute density histogram buckets from filtered record timestamps.
///
/// Returns a Vec of counts, one per bucket.
#[cfg(test)]
pub fn compute_density(timestamps: &[DateTime<Utc>], num_buckets: usize) -> Vec<usize> {
    if timestamps.is_empty() || num_buckets == 0 {
        return vec![0; num_buckets.max(1)];
    }

    let min_ts = timestamps[0];
    let max_ts = timestamps[timestamps.len() - 1];

    if min_ts == max_ts {
        // All records at same timestamp
        let mut buckets = vec![0; num_buckets];
        buckets[0] = timestamps.len();
        return buckets;
    }

    let range_ms = (max_ts - min_ts).num_milliseconds() as f64;
    let mut buckets = vec![0usize; num_buckets];

    for ts in timestamps {
        let offset_ms = (*ts - min_ts).num_milliseconds() as f64;
        let idx = ((offset_ms / range_ms) * (num_buckets as f64 - 1.0)) as usize;
        buckets[idx.min(num_buckets - 1)] += 1;
    }

    buckets
}

/// Find which bucket the cursor timestamp falls into.
#[cfg(test)]
pub fn cursor_bucket(
    cursor_ts: DateTime<Utc>,
    timestamps: &[DateTime<Utc>],
    num_buckets: usize,
) -> Option<usize> {
    if timestamps.is_empty() || num_buckets == 0 {
        return None;
    }
    let min_ts = timestamps[0];
    let max_ts = timestamps[timestamps.len() - 1];
    if min_ts == max_ts {
        return Some(0);
    }
    let range_ms = (max_ts - min_ts).num_milliseconds() as f64;
    let offset_ms = (cursor_ts - min_ts).num_milliseconds() as f64;
    let idx = ((offset_ms / range_ms) * (num_buckets as f64 - 1.0)) as usize;
    Some(idx.min(num_buckets - 1))
}

/// Compute density without collecting timestamps into a Vec.
/// Iterates filtered indices directly, looking up timestamps in records.
pub fn compute_density_indexed(
    records: &[std::sync::Arc<scouty::record::LogRecord>],
    filtered_indices: &[usize],
    num_buckets: usize,
) -> (Vec<usize>, DateTime<Utc>, DateTime<Utc>) {
    let min_ts = records[filtered_indices[0]].timestamp;
    let max_ts = records[*filtered_indices.last().unwrap()].timestamp;

    let buckets = if min_ts == max_ts {
        let mut b = vec![0usize; num_buckets];
        b[0] = filtered_indices.len();
        b
    } else {
        let range_ms = (max_ts - min_ts).num_milliseconds() as f64;
        let mut b = vec![0usize; num_buckets];
        for &i in filtered_indices {
            let ts = records[i].timestamp;
            let offset_ms = (ts - min_ts).num_milliseconds() as f64;
            let idx = ((offset_ms / range_ms) * (num_buckets as f64 - 1.0)) as usize;
            b[idx.min(num_buckets - 1)] += 1;
        }
        b
    };

    (buckets, min_ts, max_ts)
}

/// Map a bucket count to a braille height (0–4) using min-max scaling.
///
/// - Empty buckets → 0.
/// - When all non-empty buckets have the same count → 2 (mid-height,
///   distinguishable from empty but no variation to show).
/// - Otherwise, non-empty buckets are scaled 1–4 between `min_nonzero`
///   and `max_count`, stretching contrast so small variations are visible.
fn bucket_height(count: usize, min_nonzero: usize, max_count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    if min_nonzero == max_count {
        // All non-empty buckets have the same count — use mid-height
        return 2;
    }
    // Scale to 1–4 range
    let ratio = (count - min_nonzero) as f64 / (max_count - min_nonzero) as f64;
    (1.0 + ratio * 3.0).round() as usize
}

/// Render density buckets as a Braille string.
///
/// Each character encodes 2 adjacent buckets. Returns (text, cursor_char_index).
/// `cursor_bucket_idx` is the bucket index where the cursor is.
pub fn render_braille(
    buckets: &[usize],
    cursor_bucket_idx: Option<usize>,
) -> (String, Option<usize>) {
    if buckets.is_empty() {
        return (String::new(), None);
    }

    let max_count = *buckets.iter().max().unwrap_or(&1).max(&1);
    let min_nonzero = *buckets.iter().filter(|&&c| c > 0).min().unwrap_or(&0);
    let mut result = String::new();
    let mut cursor_char_idx = None;

    let mut i = 0;
    let mut char_idx = 0;
    while i < buckets.len() {
        let left = bucket_height(buckets[i], min_nonzero, max_count);
        let right = if i + 1 < buckets.len() {
            bucket_height(buckets[i + 1], min_nonzero, max_count)
        } else {
            0
        };

        result.push(braille_char(left, right));

        // Check if cursor is in this character's buckets
        if let Some(cb) = cursor_bucket_idx {
            if cb == i || (i + 1 < buckets.len() && cb == i + 1) {
                cursor_char_idx = Some(char_idx);
            }
        }

        i += 2;
        char_idx += 1;
    }

    (result, cursor_char_idx)
}
/// Default tick-mark interval for density charts (one tick every N braille chars).
pub const TICK_INTERVAL: usize = 10;

/// Compute the effective chart width (braille chars) that fits within
/// `available` display columns, accounting for tick marks every `interval`.
pub fn chart_width_for_available(available: usize, interval: usize) -> usize {
    if interval == 0 || available <= interval {
        return available;
    }
    // Exact: find largest n where n + floor((n-1)/interval) <= available.
    let mut n = (available * interval + 1) / (interval + 1);
    // Try to increase n while the total (braille + ticks) still fits.
    while n + (n.saturating_sub(1)) / interval < available {
        n += 1;
    }
    // Back off if we overshot.
    while n > 0 && n + (n.saturating_sub(1)) / interval > available {
        n -= 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick_count(braille_len: usize, interval: usize) -> usize {
        if interval == 0 || braille_len <= interval {
            return 0;
        }
        (braille_len - 1) / interval
    }
    use chrono::Duration;

    #[test]
    fn test_braille_char_empty() {
        let c = braille_char(0, 0);
        assert_eq!(c, '\u{2800}'); // empty braille
    }

    #[test]
    fn test_braille_char_full() {
        let c = braille_char(4, 4);
        assert_eq!(c, '\u{28FF}'); // full braille
    }

    #[test]
    fn test_braille_char_left_only() {
        let c = braille_char(1, 0);
        assert_eq!(c, '\u{2840}'); // only dot7
    }

    #[test]
    fn test_compute_density_empty() {
        let buckets = compute_density(&[], 10);
        assert_eq!(buckets.len(), 10);
        assert!(buckets.iter().all(|&c| c == 0));
    }

    #[test]
    fn test_compute_density_uniform() {
        let base = Utc::now();
        let timestamps: Vec<DateTime<Utc>> =
            (0..100).map(|i| base + Duration::seconds(i)).collect();
        let buckets = compute_density(&timestamps, 10);
        assert_eq!(buckets.len(), 10);
        let total: usize = buckets.iter().sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn test_render_braille_basic() {
        let buckets = vec![10, 5, 0, 8, 10, 3];
        let (text, _cursor) = render_braille(&buckets, None);
        assert_eq!(text.chars().count(), 3); // 6 buckets / 2 = 3 chars
    }

    #[test]
    fn test_cursor_bucket() {
        let base = Utc::now();
        let timestamps: Vec<DateTime<Utc>> =
            (0..100).map(|i| base + Duration::seconds(i)).collect();
        let cb = cursor_bucket(base + Duration::seconds(50), &timestamps, 10);
        assert!(cb.is_some());
        // Bucket should be roughly in the middle (4 or 5)
        let idx = cb.unwrap();
        assert!(idx >= 4 && idx <= 5, "Expected ~middle bucket, got {}", idx);
    }

    #[test]
    fn test_bucket_height_empty() {
        assert_eq!(bucket_height(0, 5, 20), 0);
    }

    #[test]
    fn test_bucket_height_min_equals_max() {
        assert_eq!(bucket_height(10, 10, 10), 2);
    }

    #[test]
    fn test_bucket_height_range() {
        // min=5, max=20 → count=5 → 1, count=20 → 4
        assert_eq!(bucket_height(5, 5, 20), 1);
        assert_eq!(bucket_height(20, 5, 20), 4);
        // mid-range: count=12 → ratio=7/15≈0.47 → 1+1.4=2.4 → 2
        assert_eq!(bucket_height(12, 5, 20), 2);
    }

    #[test]
    fn test_render_braille_contrast() {
        // Buckets with small variation should still show different heights
        let buckets = vec![10, 8, 12, 9, 11, 10, 8, 12];
        let (text, _) = render_braille(&buckets, None);
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(chars.len(), 4);
        // With min-max scaling, 8→1, 12→4, so chars should differ
        assert!(
            chars[0] != chars[1] || chars[1] != chars[2] || chars[2] != chars[3],
            "Expected varying braille chars, got all same"
        );
    }
    #[test]
    fn test_tick_count() {
        assert_eq!(tick_count(0, 10), 0);
        assert_eq!(tick_count(5, 10), 0);
        assert_eq!(tick_count(10, 10), 0);
        assert_eq!(tick_count(11, 10), 1);
        assert_eq!(tick_count(20, 10), 1);
        assert_eq!(tick_count(21, 10), 2);
        assert_eq!(tick_count(30, 10), 2);
        assert_eq!(tick_count(31, 10), 3);
    }

    #[test]
    fn test_chart_width_for_available() {
        use super::chart_width_for_available;
        // 10 columns available, interval 10 => fits 10 braille (0 ticks)
        assert_eq!(chart_width_for_available(10, 10), 10);
        // 11 columns: 10 braille + 1 tick = 11, or 11 braille + 1 tick = 12 > 11
        let w = chart_width_for_available(11, 10);
        assert!(w + tick_count(w, 10) <= 11);
        // 22 columns: 20 braille + 1 tick = 21 <= 22
        let w = chart_width_for_available(22, 10);
        assert!(w + tick_count(w, 10) <= 22);
        assert!(w >= 20);
    }
}
