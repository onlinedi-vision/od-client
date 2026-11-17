//! Selection over reported native capabilities only; no resampling or probes.
#[derive(Clone, Copy)]
pub(super) struct Range {
    pub min: u32,
    pub max: u32,
    pub channels: u16,
    pub float: bool,
}

pub(super) fn common(input: &[Range], output: &[Range]) -> Option<(usize, usize, u32)> {
    let valid = |r: &Range| r.min > 0 && r.min <= r.max && (1..=2).contains(&r.channels);
    for preferred in [Some(48000), Some(44100), None] {
        let mut best = None;
        for (i, a) in input.iter().enumerate().filter(|(_, r)| valid(r)) {
            for (o, b) in output.iter().enumerate().filter(|(_, r)| valid(r)) {
                let min = a.min.max(b.min);
                let max = a.max.min(b.max);
                let rate = preferred.unwrap_or(min);
                if min > max || !(min..=max).contains(&rate) {
                    continue;
                }
                let score = u8::from(!a.float) + u8::from(!b.float);
                if best.as_ref().is_none_or(|&(_, _, _, old)| score < old) {
                    best = Some((i, o, rate, score));
                }
            }
        }
        if let Some((i, o, rate, _)) = best {
            return Some((i, o, rate));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn r(min: u32, max: u32, channels: u16, float: bool) -> Range {
        Range {
            min,
            max,
            channels,
            float,
        }
    }
    #[test]
    fn common_rate_prefers_48k_float_and_falls_back_to_real_intersections() {
        let input = [r(44100, 48000, 1, false), r(44100, 48000, 1, true)];
        let output = [r(48000, 48000, 2, true)];
        assert_eq!(common(&input, &output), Some((1, 0, 48000)));
        assert_eq!(
            common(&[r(44100, 44100, 2, true)], &[r(44100, 96000, 1, false)]),
            Some((0, 0, 44100))
        );
        assert_eq!(
            common(&[r(32000, 32000, 1, true)], &[r(22050, 32000, 2, true)]),
            Some((0, 0, 32000))
        );
        assert!(common(&[r(44100, 44100, 1, true)], &output).is_none());
    }
    #[test]
    fn empty_zero_invalid_and_unsupported_layouts_never_produce_a_configuration() {
        assert!(common(&[], &[]).is_none());
        for range in [
            r(0, 48000, 1, true),
            r(48000, 44100, 1, true),
            r(48000, 48000, 6, true),
        ] {
            assert!(common(&[range], &[r(48000, 48000, 2, true)]).is_none());
        }
    }

    #[test]
    fn current_native_formats_require_equal_rates_without_preferring_a_new_48k_rate() {
        // CoreAudio supplies a single current format per endpoint. The shared
        // selector must reject mismatches, not select a new hardware rate.
        for rate in [32000, 44100, 48000, 96000] {
            let input = [r(rate, rate, 1, true)];
            let output = [r(rate, rate, 2, false)];
            assert_eq!(common(&input, &output), Some((0, 0, rate)));
            assert!(common(&input, &[r(rate + 1, rate + 1, 2, true)]).is_none());
        }
    }
}
