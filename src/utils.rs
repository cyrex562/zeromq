use anyhow::bail;

pub fn copy_bytes(
    src: &[u8],
    src_offset: usize,
    src_count: usize,
    dst: &mut [u8],
    dst_offset: usize,
    dst_count: usize,
) -> anyhow::Result<()> {
    if dst_count > src.len() - src_offset {
        bail!("insufficient length in source to copy destination")
    }

    for i in 0..src_count {
        dst[dst_offset + i] = src[src_offset + i];
    }

    Ok(())
}
