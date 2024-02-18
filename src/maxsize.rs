use crate::SamplingFactor;

/// The maximum size of the buffer (in bytes) required to hold a JPEG image with the given parameters.
///
/// ## Note
/// The resulting size may be larger than original image.
///
/// This implementation is direct translation of tj3JPEGBufSize from turbojpeg.
///
/// ## Example
/// ```
/// use jpeg_encoder::{max_output_size, SamplingFactor};
///
/// let w = 1920;
/// let h = 1080;
/// let buf_size: usize = max_output_size(w, h, SamplingFactor::R_4_2_0)
///     .try_into()
///     .expect("image size too large");
///
/// let mut buffer = Vec::<u8>::with_capacity(buf_size);
/// // encode image into buffer
/// ```
pub fn max_output_size(width: u16, height: u16, sampling_factor: SamplingFactor) -> u64 {
    let (mcu_width, mcu_height) = sampling_factor.mcu_size();

    let padded_w = (width as u64).next_multiple_of(mcu_width as u64);
    let padded_h = (height as u64).next_multiple_of(mcu_height as u64);

    // This term should be zero when using grayscale subsampling
    let chroma = 4 * 64 / (mcu_width as u64 * mcu_height as u64);

    padded_w * padded_h * (2 + chroma) + 2048
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_output_sizes() {
        use SamplingFactor::*;

        // validate against https://github.com/libjpeg-turbo/libjpeg-turbo/blob/26fc07c8d12cc02cf95a5ec745178f9d1916556a/turbojpeg.c#L928

        assert_eq!(2048, max_output_size(0, 0, R_4_4_4));

        assert_eq!(2432, max_output_size(1, 1, R_4_4_4));
        assert_eq!(3200, max_output_size(1, 17, R_4_4_4));
        assert_eq!(2816, max_output_size(8, 16, R_4_4_4));
        assert_eq!(5120, max_output_size(16, 32, R_4_4_4));

        assert_eq!(2560, max_output_size(1, 1, R_4_2_2));
        assert_eq!(3584, max_output_size(1, 17, R_4_2_2));
        assert_eq!(3072, max_output_size(8, 16, R_4_2_2));
        assert_eq!(4096, max_output_size(16, 32, R_4_2_2));

        assert_eq!(2816, max_output_size(1, 1, R_4_2_0));
        assert_eq!(3584, max_output_size(1, 17, R_4_2_0));
        assert_eq!(2816, max_output_size(8, 16, R_4_2_0));
        assert_eq!(3584, max_output_size(16, 32, R_4_2_0));

        assert_eq!(2816, max_output_size(1, 1, R_4_1_1));
        assert_eq!(4352, max_output_size(1, 17, R_4_1_1));
        assert_eq!(3584, max_output_size(8, 16, R_4_1_1));
        assert_eq!(5120, max_output_size(16, 32, R_4_1_1));

        assert_eq!(25769805824, max_output_size(65535, 65535, R_4_4_4));
        assert_eq!(17179871232, max_output_size(65535, 65535, R_4_2_2));
        assert_eq!(12884903936, max_output_size(65535, 65535, R_4_2_0));
    }
}
