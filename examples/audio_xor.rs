// L op R on the stereo 16-bit/44.1 input, for op in {AND, OR, XOR}. Output is one mono raw s16 file per op; verifies spirix matches native i16.
//
// Input:  /tmp/audio_stereo.s16  (raw s16le stereo, interleaved) Output: /tmp/audio_{and,or,xor}.s16

use spirix::ScalarF4E3;
use std::fs::File;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let mut bytes = Vec::new();
    File::open("/tmp/audio_stereo.s16")?.read_to_end(&mut bytes)?;
    assert!(bytes.len() % 4 == 0, "expected stereo s16 frames");
    let frames = bytes.len() / 4;

    let mut and_out = Vec::<u8>::with_capacity(frames * 2);
    let mut or_out = Vec::<u8>::with_capacity(frames * 2);
    let mut xor_out = Vec::<u8>::with_capacity(frames * 2);

    let mut and_mismatch = 0u64;
    let mut or_mismatch = 0u64;
    let mut xor_mismatch = 0u64;

    let to_i16 = |x: ScalarF4E3| -> i16 {
        let f: f32 = (&x).into();
        f.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16
    };

    for i in 0..frames {
        let l = i16::from_le_bytes([bytes[i * 4], bytes[i * 4 + 1]]);
        let r = i16::from_le_bytes([bytes[i * 4 + 2], bytes[i * 4 + 3]]);
        let ls = ScalarF4E3::from(l as f32);
        let rs = ScalarF4E3::from(r as f32);

        let s_and = to_i16(ls & rs);
        let s_or = to_i16(ls | rs);
        let s_xor = to_i16(ls ^ rs);

        if s_and != l & r { and_mismatch += 1; }
        if s_or  != l | r { or_mismatch  += 1; }
        if s_xor != l ^ r { xor_mismatch += 1; }

        and_out.extend_from_slice(&s_and.to_le_bytes());
        or_out.extend_from_slice(&s_or.to_le_bytes());
        xor_out.extend_from_slice(&s_xor.to_le_bytes());
    }

    File::create("/tmp/audio_and.s16")?.write_all(&and_out)?;
    File::create("/tmp/audio_or.s16")?.write_all(&or_out)?;
    File::create("/tmp/audio_xor.s16")?.write_all(&xor_out)?;

    println!("frames: {frames}");
    println!("AND mismatches vs native: {and_mismatch}");
    println!("OR  mismatches vs native: {or_mismatch}");
    println!("XOR mismatches vs native: {xor_mismatch}");
    Ok(())
}
