use crate::asr::Segment;

use super::split::{merge_into, overlap_ms, split_segment};

pub struct SubtitleBuffer {
    buffer_ms: u64,
    max_duration_ms: u64,
    pool: Vec<Segment>,
}

impl SubtitleBuffer {
    pub fn new(buffer_ms: u64, max_duration_ms: u64) -> Self {
        Self {
            buffer_ms,
            max_duration_ms,
            pool: Vec::new(),
        }
    }

    pub fn push(&mut self, segment: Segment) {
        for split in split_segment(segment, self.max_duration_ms) {
            self.pool.push(split);
        }
    }

    pub fn flush(&mut self, stream_position_ms: u64) -> Vec<Segment> {
        let cutoff = stream_position_ms.saturating_sub(self.buffer_ms);
        let mut ready = Vec::new();
        let mut keep = Vec::new();

        for seg in self.pool.drain(..) {
            if seg.end_ms <= cutoff {
                if let Some(last) = ready.last_mut()
                    && overlap_ms(last, &seg)
                {
                    merge_into(last, &seg);
                    continue;
                }
                ready.push(seg);
            } else {
                keep.push(seg);
            }
        }

        self.pool = keep;
        ready
    }

    pub fn drain(&mut self) -> Vec<Segment> {
        std::mem::take(&mut self.pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_push_splits_long_segments() {
        println!("Описание: push() автоматически дробит длинные сегменты через split_segment");
        let mut buf = SubtitleBuffer::new(2000, 3000);
        buf.push(Segment {
            start_ms: 0,
            end_ms: 10000,
            text: "a b c d".into(),
        });
        assert_eq!(buf.drain().len(), 4);
    }

    #[test]
    fn buffer_flush_respects_buffer_ms() {
        println!("Описание: flush() сдвигает cutoff на buffer_ms назад, неготовые сегменты не выгружаются");
        let mut buf = SubtitleBuffer::new(2000, 10000);
        buf.push(Segment {
            start_ms: 1000,
            end_ms: 2000,
            text: "hello".into(),
        });
        buf.push(Segment {
            start_ms: 3000,
            end_ms: 4000,
            text: "world".into(),
        });

        // stream at 3000ms → cutoff = 1000 → first seg (end_ms=2000) isn't flushed yet
        let flushed = buf.flush(3000);
        assert_eq!(flushed.len(), 0);

        // stream at 5000ms → cutoff = 3000 → first seg's end_ms (2000) ≤ 3000 → flushed
        let flushed = buf.flush(5000);
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].text, "hello");
    }

    #[test]
    fn buffer_flush_merges_overlapping_segments() {
        println!("Описание: перекрывающиеся по времени сегменты сливаются в один при flush()");
        let mut buf = SubtitleBuffer::new(2000, 10000);
        buf.push(Segment {
            start_ms: 1000,
            end_ms: 2500,
            text: "hello".into(),
        });
        buf.push(Segment {
            start_ms: 2000,
            end_ms: 3500,
            text: "world".into(),
        });

        let flushed = buf.flush(10000);
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].start_ms, 1000);
        assert_eq!(flushed[0].end_ms, 3500);
        assert!(flushed[0].text.contains("hello"));
        assert!(flushed[0].text.contains("world"));
    }

    #[test]
    fn buffer_flush_keeps_non_ready_segments() {
        println!("Описание: сегменты, не достигшие cutoff, остаются в буфере после flush()");
        let mut buf = SubtitleBuffer::new(2000, 10000);
        buf.push(Segment {
            start_ms: 1000,
            end_ms: 2000,
            text: "old".into(),
        });
        buf.push(Segment {
            start_ms: 9000,
            end_ms: 10000,
            text: "recent".into(),
        });

        // cutoff = 3000 → only first seg ready
        let flushed = buf.flush(5000);
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].text, "old");

        // remaining should have "recent"
        let remaining = buf.drain();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].text, "recent");
    }

    #[test]
    fn drain_empties_buffer() {
        println!("Описание: после drain() буфер становится пустым");
        let mut buf = SubtitleBuffer::new(2000, 10000);
        buf.push(Segment {
            start_ms: 0,
            end_ms: 1000,
            text: "x".into(),
        });
        assert_eq!(buf.drain().len(), 1);
        assert!(buf.drain().is_empty());
    }
}
