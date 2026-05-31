use crate::asr::Segment;

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

pub(super) fn overlap_ms(a: &Segment, b: &Segment) -> bool {
    a.end_ms >= b.start_ms && b.end_ms >= a.start_ms
}

pub(super) fn merge_into(a: &mut Segment, b: &Segment) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_segment_short_enough() {
        println!("Описание: сегмент короче max_duration не дробится, возвращается как есть");
        let seg = Segment {
            start_ms: 1000,
            end_ms: 3000,
            text: "hello world".into(),
        };
        let result = split_segment(seg.clone(), 5000);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_ms, 1000);
        assert_eq!(result[0].end_ms, 3000);
        assert_eq!(result[0].text, "hello world");
    }

    #[test]
    fn split_segment_zero_max_duration() {
        println!("Описание: при max_duration=0 сегмент не дробится (защита от деления на ноль)");
        let seg = Segment {
            start_ms: 0,
            end_ms: 5000,
            text: "a b c".into(),
        };
        let result = split_segment(seg.clone(), 0);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn split_segment_needs_splitting() {
        println!("Описание: сегмент длиннее max_duration дробится по словам, временные границы сохраняются");
        let seg = Segment {
            start_ms: 0,
            end_ms: 10000,
            text: "one two three four".into(),
        };
        let result = split_segment(seg, 3000);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].text, "one");
        assert_eq!(result[1].text, "two");
        assert_eq!(result[2].text, "three");
        assert_eq!(result[3].text, "four");
        assert_eq!(result[0].start_ms, 0);
        assert_eq!(result[3].end_ms, 10000);
    }

    #[test]
    fn split_segment_exact_boundary() {
        println!("Описание: сегмент ровно по границе max_duration не дробится");
        let seg = Segment {
            start_ms: 0,
            end_ms: 5000,
            text: "a b".into(),
        };
        let result = split_segment(seg, 5000);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn overlap_ms_detects_overlap() {
        println!("Описание: overlap_ms() возвращает true для пересекающихся по времени сегментов (симметрично)");
        let a = Segment {
            start_ms: 1000,
            end_ms: 3000,
            text: "a".into(),
        };
        let b = Segment {
            start_ms: 2500,
            end_ms: 4000,
            text: "b".into(),
        };
        assert!(overlap_ms(&a, &b));
        assert!(overlap_ms(&b, &a));
    }

    #[test]
    fn overlap_ms_no_overlap() {
        println!("Описание: overlap_ms() возвращает false для непересекающихся сегментов");
        let a = Segment {
            start_ms: 1000,
            end_ms: 2000,
            text: "a".into(),
        };
        let b = Segment {
            start_ms: 3000,
            end_ms: 4000,
            text: "b".into(),
        };
        assert!(!overlap_ms(&a, &b));
        assert!(!overlap_ms(&b, &a));
    }

    #[test]
    fn merge_into_extends_bounds_and_text() {
        println!("Описание: merge_into() расширяет временные границы и склеивает тексты через пробел");
        let mut a = Segment {
            start_ms: 2000,
            end_ms: 3000,
            text: "foo".into(),
        };
        let b = Segment {
            start_ms: 1000,
            end_ms: 4000,
            text: "bar".into(),
        };
        merge_into(&mut a, &b);
        assert_eq!(a.start_ms, 1000);
        assert_eq!(a.end_ms, 4000);
        assert_eq!(a.text, "foo bar");
    }
}
