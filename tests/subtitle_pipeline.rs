use std::fs;

use audiosub::asr::Segment;
use audiosub::subtitle::{SubtitleBuffer, SubtitleOutput, create_writer, split_segment};

fn segments_for_test() -> Vec<Segment> {
    vec![
        Segment {
            start_ms: 1000,
            end_ms: 3000,
            text: "hello world".into(),
        },
        Segment {
            start_ms: 3500,
            end_ms: 5000,
            text: "second phrase".into(),
        },
        Segment {
            start_ms: 6000,
            end_ms: 8000,
            text: "third line".into(),
        },
    ]
}

#[test]
fn srt_writer_produces_valid_srt_content() {
    let mut output = Vec::<u8>::new();
    let segments = segments_for_test();
    let mut writer = create_writer("srt");
    writer.write_header(&mut output).unwrap();
    for (i, seg) in segments.iter().enumerate() {
        writer.write_segment(&mut output, seg, i + 1).unwrap();
    }
    writer.write_footer(&mut output).unwrap();

    let content = String::from_utf8(output).unwrap();
    assert!(content.contains("00:00:01,000 --> 00:00:03,000"));
    assert!(content.contains("hello world"));
    assert!(content.contains("00:00:03,500 --> 00:00:05,000"));
    assert!(content.contains("second phrase"));
    assert!(content.contains("00:00:06,000 --> 00:00:08,000"));
    assert!(content.contains("third line"));
}

#[test]
fn vtt_writer_produces_valid_vtt_content() {
    let mut output = Vec::<u8>::new();
    let segments = segments_for_test();
    let mut writer = create_writer("vtt");
    writer.write_header(&mut output).unwrap();
    for (i, seg) in segments.iter().enumerate() {
        writer.write_segment(&mut output, seg, i + 1).unwrap();
    }
    writer.write_footer(&mut output).unwrap();

    let content = String::from_utf8(output).unwrap();
    assert!(content.starts_with("WEBVTT"));
    assert!(content.contains("00:00:01.000 --> 00:00:03.000"));
    assert!(content.contains("hello world"));
    assert!(content.contains("00:00:03.500 --> 00:00:05.000"));
    assert!(content.contains("second phrase"));
}

#[test]
fn subtitle_output_writes_file_on_disk() {
    let dir = fs::canonicalize(".").unwrap();
    // Use a unique temp name to avoid collision
    let path = dir.join(format!("test_output_{}.srt", std::process::id()));
    // Clean up from previous runs
    let _ = fs::remove_file(&path);

    let mut out = SubtitleOutput::create(&path, "srt").unwrap();
    let segments = segments_for_test();
    out.append_all(&segments).unwrap();
    out.close().unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("hello world"));
    assert!(content.contains("second phrase"));
    assert!(content.contains("third line"));

    fs::remove_file(&path).unwrap();
}

#[test]
fn subtitle_output_vtt_writes_file() {
    let dir = fs::canonicalize(".").unwrap();
    let path = dir.join(format!("test_output_vtt_{}.vtt", std::process::id()));
    let _ = fs::remove_file(&path);

    let mut out = SubtitleOutput::create(&path, "vtt").unwrap();
    out.append(&Segment {
        start_ms: 0,
        end_ms: 1000,
        text: "caption".into(),
    })
    .unwrap();
    out.close().unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("WEBVTT"));
    assert!(content.contains("caption"));

    fs::remove_file(&path).unwrap();
}

#[test]
fn buffer_to_output_integration() {
    let dir = fs::canonicalize(".").unwrap();
    let path = dir.join(format!("test_buffer_out_{}.srt", std::process::id()));
    let _ = fs::remove_file(&path);

    let mut buf = SubtitleBuffer::new(2000, 5000);
    buf.push(Segment {
        start_ms: 0,
        end_ms: 2000,
        text: "first".into(),
    });
    buf.push(Segment {
        start_ms: 2500,
        end_ms: 4000,
        text: "second".into(),
    });

    // Flush with enough margin
    let ready = buf.flush(8000);
    assert!(!ready.is_empty());

    let mut out = SubtitleOutput::create(&path, "srt").unwrap();
    out.append_all(&ready).unwrap();
    out.close().unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("first"));
    assert!(content.contains("second"));

    fs::remove_file(&path).unwrap();
}

#[test]
fn split_segment_preserves_all_text() {
    let seg = Segment {
        start_ms: 0,
        end_ms: 10000,
        text: "one two three four five six seven eight nine ten".into(),
    };
    let parts = split_segment(seg, 3000);
    assert!(parts.len() >= 3);
    let combined: String = parts.iter().map(|s| s.text.clone()).collect::<Vec<_>>().join(" ");
    assert!(combined.contains("one"));
    assert!(combined.contains("ten"));
}

#[test]
fn buffer_empty_flush_returns_empty_vec() {
    let mut buf = SubtitleBuffer::new(2000, 5000);
    let flushed = buf.flush(1000);
    assert!(flushed.is_empty());
}

#[test]
fn buffer_drain_returns_all_without_cutoff() {
    let mut buf = SubtitleBuffer::new(2000, 5000);
    buf.push(Segment {
        start_ms: 0,
        end_ms: 1000,
        text: "a".into(),
    });
    buf.push(Segment {
        start_ms: 1000,
        end_ms: 2000,
        text: "b".into(),
    });
    let all = buf.drain();
    assert_eq!(all.len(), 2);
    assert!(buf.drain().is_empty());
}
