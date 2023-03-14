pub fn copy_bytes(dest: &mut[u8], dest_offset: usize, src: &[u8], src_offset: usize, count: usize) {
    for i in 0 .. count {
        dest[dest_offset+i] = src[src_offset+i]
    }
}