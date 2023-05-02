use std::mem;

pub fn copy_bytes(
    dest: &mut [u8],
    dest_offset: usize,
    src: &[u8],
    src_offset: usize,
    count: usize,
) {
    for i in 0..count {
        dest[dest_offset + i] = src[src_offset + i]
    }
}

pub fn set_bytes(dest: &mut [u8], offset: usize, val: u8, count: usize) {
    let real_count = usize::min(dest.len() - offset, count);
    for i in 0..real_count {
        dest[i] = val;
    }
}

pub fn cmp_bytes(a: &[u8], offset_a: usize, b: &[u8], offset_b: usize, count: usize) -> u8 {
    let mut real_count = usize::min(a.len() - offset_a, b.len() - offset_b);
    real_count = usize::min(real_count, count);
    for i in 0..real_count {
        if a[offset_a + i] != b[offset_b + i] {
            return 1;
        }
    }
    return 0;
}

pub fn put_u8(dest: &mut [u8], dest_offset: usize, src: u8) {
    dest[dest_offset] = src;
}

pub fn get_u8(src: &[u8], src_offset: usize) -> u8 {
    src[src_offset]
}

pub fn put_u16(dest: &mut [u8], dest_offset: usize, src: u16) {
    let src_bytes: [u8; 2] = src.to_le_bytes();
    for i in 0..mem::size_of::<src> {
        dest[dest_offset + i] = src_bytes[i]
    }
}

pub fn get_u16(src: &[u8], src_offset: usize) -> u16 {
    let out_bytes: [u8; 2] = [0; 2];
    out_bytes.clone_from_slice(src + src_offset);
    u16::from_le_bytes(out_bytes)
}

pub fn put_u32(dest: &mut [u8], dest_offset: usize, src: u32) {
    let src_bytes: [u8; 4] = src.to_le_bytes();
    for i in 0..mem::size_of::<src>() {
        dest[dest_offset + i] = src_bytes[i]
    }
}

pub fn get_u32(src: &[u8], src_offset: usize) -> u32 {
    let out_bytes: [u8; 4] = [0; 4];
    out_bytes.clone_from_slice(src + src_offset);
    u32::from_le_bytes(out_bytes)
}

pub fn put_u64(dest: &mut [u8], dest_offset: usize, src: u64) {
    let src_bytes: [u8; 8] = src.to_le_bytes();
    for i in 0..mem::size_of::<src>() {
        dest[dest_offset + i] = src_bytes[i]
    }
}

pub fn get_u64(src: &[u8], src_offset: usize) -> u64 {
    let out_bytes: [u8; 8] = [0; 8];
    out_bytes.clone_from_slice(src + src_offset);
    u64::from_le_bytes(out_bytes)
}
