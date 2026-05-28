use crate::asr::Segment;

pub struct SubtitleBuffer {
    buffer_ms: u64,
    pool: Vec<Segment>,
}

impl SubtitleBuffer {
    pub fn new(buffer_ms: u64) -> Self {
        Self {
            buffer_ms,
            pool: Vec::new(),
        }
    }

    pub fn push(&mut self, segment: Segment) {
        self.pool.push(segment);
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
