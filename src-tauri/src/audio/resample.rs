use super::TARGET_SAMPLE_RATE;

pub fn resample_to_16k(input: &[f32], input_rate: u32) -> Vec<i16> {
    if input.is_empty() {
        return Vec::new();
    }

    if input_rate == TARGET_SAMPLE_RATE {
        return input.iter().map(|&s| f32_to_i16(s)).collect();
    }

    let ratio = input_rate as f64 / TARGET_SAMPLE_RATE as f64;
    let output_len = ((input.len() as f64) / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = (i as f64 * ratio) as usize;
        let sample = input.get(src_idx).copied().unwrap_or(0.0);
        output.push(f32_to_i16(sample));
    }

    output
}

fn f32_to_i16(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    (clamped * i16::MAX as f32) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_produces_shorter_output_for_downsample() {
        let input: Vec<f32> = (0..48000).map(|i| (i as f32).sin()).collect();
        let out = resample_to_16k(&input, 48000);
        assert!(out.len() <= 16001);
        assert!(out.len() >= 15999);
    }
}
