use crate::asr::Segment;

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

pub fn split_segment(seg: Segment, max_duration_ms: u64) -> Vec<Segment> {
    if max_duration_ms == 0 || seg.end_ms - seg.start_ms <= max_duration_ms {
        return vec![seg];
    }

    let duration = seg.end_ms - seg.start_ms;
    let count = duration.div_ceil(max_duration_ms);
    let chunk_duration = duration / count;

    let mut result = Vec::with_capacity(count as usize);
    let mut offset = seg.start_ms;

    let words: Vec<&str> = seg.text.split_whitespace().collect();
    let words_per_chunk = if count > 0 {
        words.len() / count as usize
    } else {
        words.len()
    };

    for i in 0..count {
        let chunk_end = if i == count - 1 {
            seg.end_ms
        } else {
            offset + chunk_duration
        };
        let start_idx = i as usize * words_per_chunk;
        let end_idx = if i == count - 1 {
            words.len()
        } else {
            start_idx + words_per_chunk
        };

        let text = words[start_idx..end_idx].join(" ");
        result.push(Segment {
            start_ms: offset,
            end_ms: chunk_end,
            text,
        });
        offset = chunk_end;
    }

    result
}

fn overlap_ms(a: &Segment, b: &Segment) -> bool {
    a.end_ms >= b.start_ms && b.end_ms >= a.start_ms
}

fn merge_into(a: &mut Segment, b: &Segment) {
    if b.start_ms < a.start_ms {
        a.start_ms = b.start_ms;
    }
    if b.end_ms > a.end_ms {
        a.end_ms = b.end_ms;
    }
    if !a.text.is_empty() && !b.text.is_empty() {
        a.text.push(' ');
    }
    a.text.push_str(&b.text);
}
