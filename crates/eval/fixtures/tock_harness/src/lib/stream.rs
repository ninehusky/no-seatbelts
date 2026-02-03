use capsules_core::stream;
use capsules_core::stream::SResult;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_SRESULT_IS_DONE: fn(&SResult) -> bool = stream_sresult_is_done_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_SRESULT_IS_ERR: fn(&SResult) -> bool = stream_sresult_is_err_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_SRESULT_DONE: fn(SResult) -> Option<(usize, ())> = stream_sresult_done_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_SRESULT_NEEDED: fn(SResult) -> Option<usize> = stream_sresult_needed_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_SRESULT_ERR: fn(SResult) -> Option<()> = stream_sresult_err_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_ENCODE_U8: fn(&mut [u8], u8) -> SResult = stream_encode_u8_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_ENCODE_U16: fn(&mut [u8], u16) -> SResult = stream_encode_u16_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_ENCODE_U32: fn(&mut [u8], u32) -> SResult = stream_encode_u32_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_ENCODE_BYTES: fn(&mut [u8], &[u8]) -> SResult = stream_encode_bytes_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_ENCODE_BYTES_BE: fn(&mut [u8], &[u8]) -> SResult =
    stream_encode_bytes_be_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_DECODE_U8: fn(&[u8]) -> SResult<u8> = stream_decode_u8_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_DECODE_U16: fn(&[u8]) -> SResult<u16> = stream_decode_u16_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_DECODE_U32: fn(&[u8]) -> SResult<u32> = stream_decode_u32_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_DECODE_BYTES: fn(&[u8], &mut [u8]) -> SResult = stream_decode_bytes_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_STREAM_DECODE_BYTES_BE: fn(&[u8], &mut [u8]) -> SResult =
    stream_decode_bytes_be_shim;

#[inline(never)]
fn stream_encode_u8_shim(buffer: &mut [u8], value: u8) -> SResult {
    stream::encode_u8(buffer, value)
}

#[inline(never)]
fn stream_encode_u16_shim(buffer: &mut [u8], value: u16) -> SResult {
    stream::encode_u16(buffer, value)
}

#[inline(never)]
fn stream_encode_u32_shim(buffer: &mut [u8], value: u32) -> SResult {
    stream::encode_u32(buffer, value)
}

#[inline(never)]
fn stream_encode_bytes_shim(buffer: &mut [u8], value: &[u8]) -> SResult {
    stream::encode_bytes(buffer, value)
}

#[inline(never)]
fn stream_encode_bytes_be_shim(buffer: &mut [u8], value: &[u8]) -> SResult {
    stream::encode_bytes_be(buffer, value)
}

#[inline(never)]
fn stream_decode_u8_shim(buffer: &[u8]) -> SResult<u8> {
    stream::decode_u8(buffer)
}

#[inline(never)]
fn stream_decode_u16_shim(buffer: &[u8]) -> SResult<u16> {
    stream::decode_u16(buffer)
}

#[inline(never)]
fn stream_decode_u32_shim(buffer: &[u8]) -> SResult<u32> {
    stream::decode_u32(buffer)
}

#[inline(never)]
fn stream_decode_bytes_shim(buffer: &[u8], output: &mut [u8]) -> SResult {
    stream::decode_bytes(buffer, output)
}

#[inline(never)]
fn stream_decode_bytes_be_shim(buffer: &[u8], output: &mut [u8]) -> SResult {
    stream::decode_bytes_be(buffer, output)
}

#[inline(never)]
fn stream_sresult_is_done_shim(sresult: &SResult) -> bool {
    sresult.is_done()
}

#[inline(never)]
fn stream_sresult_is_err_shim(sresult: &SResult) -> bool {
    sresult.is_err()
}

#[inline(never)]
fn stream_sresult_done_shim(sresult: SResult) -> Option<(usize, ())> {
    sresult.done()
}

#[inline(never)]
fn stream_sresult_needed_shim(sresult: SResult) -> Option<usize> {
    sresult.needed()
}

#[inline(never)]
fn stream_sresult_err_shim(sresult: SResult) -> Option<()> {
    sresult.err()
}
