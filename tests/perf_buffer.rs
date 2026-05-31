use std::time::Instant;

use audiosub::asr::Segment;
use audiosub::subtitle::{SubtitleBuffer, create_writer, split_segment};

const ITERATIONS: u32 = 1000;

#[test]
fn perf_split_segment_300s_20_words() {
    let seg = Segment {
        start_ms: 0,
        end_ms: 300_000,
        text: "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty".into(),
    };

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let result = split_segment(seg.clone(), 10_000);
        assert!(result.len() > 1);
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / ITERATIONS as u128;
    println!("split_segment_300s_20_words: {avg_ns} ns/iter, total {elapsed:?} for {ITERATIONS} iters");
    // Should be well under 10µs per call
    assert!(avg_ns < 20_000, "split_segment too slow: {avg_ns} ns/iter");
}

#[test]
fn perf_split_segment_no_split() {
    let seg = Segment {
        start_ms: 1000,
        end_ms: 3000,
        text: "hello world".into(),
    };

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let result = split_segment(seg.clone(), 5000);
        assert_eq!(result.len(), 1);
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / ITERATIONS as u128;
    println!(
        "split_segment_no_split: {avg_ns} ns/iter, total {elapsed:?}",
        elapsed = elapsed
    );
    assert!(avg_ns < 2000, "split_segment no-split too slow: {avg_ns} ns/iter");
}

#[test]
fn perf_buffer_push_1000_segments() {
    let seg = Segment {
        start_ms: 0,
        end_ms: 1000,
        text: "test phrase".into(),
    };

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut buf = SubtitleBuffer::new(2000, 5000);
        for _ in 0..1000 {
            buf.push(seg.clone());
        }
        assert_eq!(buf.drain().len(), 1000);
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / ITERATIONS as u128;
    println!(
        "buffer_push_1000: {avg_ns} ns/iter, total {elapsed:?}",
        elapsed = elapsed
    );
    // ~1000 * ITERATIONS pushes, allow some time
    assert!(avg_ns < 5_000_000, "buffer push too slow: {avg_ns} ns/iter");
}

#[test]
fn perf_buffer_flush_500_ready_500_pending() {
    let start = Instant::now();
    for _ in 0..100 {
        let mut buf = SubtitleBuffer::new(2000, 5000);
        // Non-overlapping segments: width 80ms, gap 120ms → no merge
        for i in 0..500 {
            buf.push(Segment {
                start_ms: i * 200,
                end_ms: i * 200 + 80,
                text: "ready".into(),
            });
        }
        for i in 0..500 {
            buf.push(Segment {
                start_ms: 200_000 + i * 200,
                end_ms: 200_000 + i * 200 + 80,
                text: "pending".into(),
            });
        }
        // cutoff = 8000 - 2000 = 6000 → ready segs with end_ms ≤ 6000
        let flushed = buf.flush(8000);
        // ready seg[0] end_ms=80, ready seg[29] end_ms=29*200+80=5880 → ready seg[30] end_ms=6080 > 6000
        // So 30 segs flushed (indices 0-29)
        assert_eq!(flushed.len(), 30, "expected 30 ready segments, got {}", flushed.len());
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() / 100;
    println!(
        "buffer_flush_500+500: {avg_us} µs/iter, total {elapsed:?}",
        elapsed = elapsed
    );
    assert!(avg_us < 50_000, "buffer flush too slow: {avg_us} µs");
}

#[test]
fn perf_srt_write_segment() {
    let seg = Segment {
        start_ms: 1_234_567,
        end_ms: 2_345_678,
        text: "This is a sample subtitle line with several words".into(),
    };

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut buf = Vec::<u8>::new();
        let mut writer = create_writer("srt");
        writer.write_segment(&mut buf, &seg, 1).unwrap();
        assert!(buf.len() > 20);
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / ITERATIONS as u128;
    println!(
        "srt_write_segment: {avg_ns} ns/iter, total {elapsed:?}",
        elapsed = elapsed
    );
    assert!(avg_ns < 5000, "srt write too slow: {avg_ns} ns/iter");
}

#[test]
fn perf_vtt_write_segment() {
    let seg = Segment {
        start_ms: 1_234_567,
        end_ms: 2_345_678,
        text: "This is a sample subtitle line with several words".into(),
    };

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut buf = Vec::<u8>::new();
        let mut writer = create_writer("vtt");
        writer.write_segment(&mut buf, &seg, 1).unwrap();
        assert!(buf.len() > 20);
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / ITERATIONS as u128;
    println!(
        "vtt_write_segment: {avg_ns} ns/iter, total {elapsed:?}",
        elapsed = elapsed
    );
    assert!(avg_ns < 5000, "vtt write too slow: {avg_ns} ns/iter");
}
