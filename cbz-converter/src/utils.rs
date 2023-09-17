static BASE_32_SCALELST: [u64; 8] = [
    1,
    32,
    1_024,
    32_768,
    1_048_576,
    33_554_432,
    1_073_741_824,
    34_359_738_368,
];

// Adapted from https://github.com/iscc/mobi/blob/cb9ad8fd261f23d669c7e2d56a7a34b5aff29036/mobi/mobi_utils.py#L216
// The base32 crate didn't work in our case
pub fn base_32(bytes: &[u8]) -> u64 {
    let mut value = 0;
    let mut scale = 0;
    for (i, byte) in bytes.iter().rev().enumerate() {
        let v = if byte.is_ascii_digit() {
            byte - b'0'
        } else {
            byte - b'A' + 10
        };
        scale = BASE_32_SCALELST.get(i).copied().unwrap_or(scale * 32);
        if v != 0 {
            value += (u64::from(v)) * scale;
        }
    }
    value
}
