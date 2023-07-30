use anyhow::bail;
use libc::c_void;

pub fn copy_bytes(
    src: &[u8],
    src_offset: usize,
    src_count: usize,
    dst: &mut [u8],
    dst_offset: usize,
) -> anyhow::Result<()> {
    if dst.len() - dst_offset < src_count {
        bail!("insufficient length in source to copy destination")
    }

    for i in 0..src_count {
        dst[dst_offset + i] = src[src_offset + i];
    }

    Ok(())
}

pub unsafe fn copy_bytes_void(
    src: *const c_void,
    src_offset: usize,
    src_count: usize,
    dst: *mut c_void,
    dst_offset: usize,
    dst_len: usize,
) -> anyhow::Result<()> {
    if dst_len - dst_offset < src_count {
        bail!("insufficient length in source to copy destination")
    }

    for i in 0..src_count {
        *dst.add(dst_offset+i) = *src.add(src_offset + i);
    }

    Ok(())
}

pub fn get_errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap()
}