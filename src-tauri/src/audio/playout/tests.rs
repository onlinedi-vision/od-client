use super::*;

fn fixture(rate: u32, channels: u16) -> (Producer, Consumer, Arc<Stats>) {
    let stats = Arc::new(Stats::default());
    let (tx, rx) = queue(rate, channels, Arc::clone(&stats)).unwrap();
    (tx, rx, stats)
}
fn render(rx: &mut Consumer, frames: usize, now: i64) -> Vec<[f32; 2]> {
    let mut result = vec![[99.0; 2]; frames];
    rx.render(frames, now, |i, frame| result[i] = frame);
    result
}

#[test]
fn storage_includes_timestamps_and_never_exceeds_40ms_or_64kib() {
    for rate in [44100, 48000, 192000, 768000] {
        for channels in [1, 2] {
            let (tx, _, _) = fixture(rate, channels);
            let capacity = tx.shared.slots.len();
            assert!(capacity > 0 && capacity * std::mem::size_of::<Frame>() <= MAX_BYTES);
            assert!(capacity as u64 * 1000 <= u64::from(rate) * 40);
        }
    }
    assert!(queue(0, 1, Arc::new(Stats::default())).is_err());
    assert!(queue(48000, 6, Arc::new(Stats::default())).is_err());
}

#[test]
fn mono_stereo_mapping_gain_and_underrun_silence_are_explicit() {
    let (mut tx, mut rx, stats) = fixture(48000, 2);
    tx.push([1.0, -1.0].into_iter(), 1, 0);
    assert_eq!(
        render(&mut rx, 3, 1_000_000),
        vec![[0.1; 2], [-0.1; 2], [0.0; 2]]
    );
    assert_eq!(stats.snapshot().underruns, 1);
    let (mut tx, mut rx, _) = fixture(48000, 1);
    tx.push([1.0, -1.0, 0.4, 0.6].into_iter(), 2, 0);
    assert_eq!(render(&mut rx, 2, 1_000_000), vec![[0.0; 2], [0.05, 0.0]]);
}

#[test]
fn stereo_is_preserved_without_nan_or_peak_amplification() {
    let (mut tx, mut rx, _) = fixture(48000, 2);
    tx.push([1.0, -1.0, f32::NAN, 10.0].into_iter(), 2, 0);
    assert_eq!(render(&mut rx, 2, 1_000_000), vec![[0.1, -0.1], [0.0, 0.1]]);
    assert!(output_sample(f32::INFINITY).is_finite());
}

#[test]
fn overflow_refuses_new_frames_then_consumer_alone_trims_to_10ms() {
    let (mut tx, mut rx, stats) = fixture(48000, 1);
    let capacity = tx.shared.slots.len();
    tx.push((0..capacity).map(|i| i as f32 / capacity as f32), 1, 0);
    let read = tx.shared.read.load(Ordering::Acquire);
    tx.push([1.0; 512].into_iter(), 1, 40_000_000);
    assert_eq!(tx.shared.read.load(Ordering::Acquire), read); // producer never steals ownership
    let skip = capacity - tx.shared.target;
    let result = render(&mut rx, 1, 40_000_000);
    assert!((result[0][0] - output_sample(skip as f32 / capacity as f32)).abs() < 1e-6);
    let d = stats.snapshot();
    assert_eq!(d.queued_frames, 479);
    assert_eq!(d.overflows, 1);
    assert_eq!(d.dropped_frames, 512 + skip as u32);
    assert_eq!(d.high_water_frames, capacity as u32);
}

#[test]
fn age_discards_even_a_tiny_queue_after_interruption_and_never_replays_it() {
    let (mut tx, mut rx, stats) = fixture(48000, 1);
    tx.push([1.0; 2].into_iter(), 1, 0);
    assert_eq!(render(&mut rx, 4, 100_000_000), vec![[0.0; 2]; 4]);
    assert_eq!(stats.snapshot().expired_frames, 2);
    tx.push([0.5; 2].into_iter(), 1, 200_000_000);
    assert_eq!(
        render(&mut rx, 3, 201_000_000),
        vec![[0.05, 0.0], [0.05, 0.0], [0.0; 2]]
    );
    tx.push([1.0].into_iter(), 1, 400_000_000); // invalid future capture clock
    assert_eq!(render(&mut rx, 1, 300_000_000), vec![[0.0; 2]]);
    assert_eq!(stats.snapshot().expired_frames, 3);
}

#[test]
fn non_power_of_two_ring_and_wrapping_counters_keep_fifo_order() {
    let (mut tx, mut rx, stats) = fixture(48000, 1);
    let count = u64::MAX - 2;
    let position = tx.shared.slots.len() - 2;
    tx.cursor = Cursor { count, position };
    rx.cursor = Cursor { count, position };
    tx.shared.written.store(count, Ordering::Release);
    tx.shared.read.store(count, Ordering::Release);
    tx.push([1.0; 8].into_iter(), 1, 0);
    assert_eq!(render(&mut rx, 8, 1_000_000), vec![[0.1, 0.0]; 8]);
    assert_eq!(stats.snapshot().queued_frames, 0);
    drop(rx);
    assert_eq!(stats.snapshot().queued_frames, 0);
}

#[test]
fn independent_callbacks_share_no_locks_and_preserve_fifo_under_concurrency() {
    let (mut tx, mut rx, stats) = fixture(48000, 1);
    const BLOCK: usize = 32;
    const BLOCKS: usize = 400;
    let started = Instant::now();
    let value = |i: usize| (i % 1000 + 1) as f32 / 1024.0;
    std::thread::scope(|scope| {
        scope.spawn(move || {
            for block in 0..BLOCKS {
                while tx
                    .cursor
                    .count
                    .wrapping_sub(tx.shared.read.load(Ordering::Acquire))
                    as usize
                    > tx.shared.slots.len() - BLOCK
                {
                    assert!(started.elapsed() < Duration::from_secs(3));
                    std::thread::yield_now();
                }
                tx.push(
                    (block * BLOCK..(block + 1) * BLOCK).map(value),
                    1,
                    -1_000_000,
                );
            }
        });
        scope.spawn(move || {
            for block in 0..BLOCKS {
                while rx
                    .shared
                    .written
                    .load(Ordering::Acquire)
                    .wrapping_sub(rx.cursor.count)
                    < BLOCK as u64
                {
                    assert!(started.elapsed() < Duration::from_secs(3));
                    std::thread::yield_now();
                }
                rx.render(BLOCK, 0, |i, frame| {
                    assert_eq!(frame[0], output_sample(value(block * BLOCK + i)))
                });
            }
        });
    });
    assert_eq!(stats.snapshot().dropped_frames, 0);
    assert_eq!(stats.snapshot().underruns, 0);
}

#[test]
fn tone_is_440hz_bounded_to_one_second_low_gain_and_silent_when_cancelled() {
    let mut tone = Signal::Tone {
        rate: 48000,
        frame: 0,
    };
    let mut crossings = 0;
    let mut previous = 0.0;
    tone.render(96000, 0, true, |i, frame| {
        assert!(frame[0].abs() <= 0.1 && frame[0] == frame[1]);
        if i >= 48000 {
            assert_eq!(frame[0], 0.0);
        }
        if i > 0 && previous <= 0.0 && frame[0] > 0.0 {
            crossings += 1;
        }
        previous = frame[0];
    });
    assert_eq!(crossings, 440);
    tone.render(512, 0, false, |_, frame| assert_eq!(frame, [0.0; 2]));
}
