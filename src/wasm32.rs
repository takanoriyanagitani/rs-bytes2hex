use std::io;

use io::BufRead;
use io::ErrorKind;

use io::BufWriter;
use io::Write;

use core::arch::wasm32::v128;

use core::arch::wasm32::u8x16;
use core::arch::wasm32::u8x16_shuffle;
use core::arch::wasm32::u8x16_splat;

use core::arch::wasm32::u64x2;
use core::arch::wasm32::u64x2_extract_lane;

use core::arch::wasm32::u16x8_shr;
use core::arch::wasm32::u16x8_splat;

use core::arch::wasm32::v128_and;
use core::arch::wasm32::v128_bitselect;

use core::arch::wasm32::i8x16_swizzle;

pub fn v2u(v: v128) -> u128 {
    let lo: u64 = u64x2_extract_lane::<0>(v);
    let hi: u64 = u64x2_extract_lane::<1>(v);

    let l: u128 = lo.into();
    let h: u128 = hi.into();

    (h << 64) | l
}

pub fn v2u_fast(v: v128) -> u128 {
    #[allow(unsafe_code)]
    unsafe {
        std::mem::transmute::<v128, u128>(v)
    }
}

#[target_feature(enable = "simd128,relaxed-simd")]
pub fn bytes2hex(bytes: [u8; 8]) -> [u8; 16] {
    let lo: u64 = u64::from_be_bytes(bytes);
    let hi: u64 = 0;
    let v: v128 = u64x2(lo, hi);

    let dup: v128 = u8x16_shuffle::<0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7>(v, v);
    let shifted: v128 = u16x8_shr(dup, 4);
    let mask: v128 = u16x8_splat(0xFF00);
    let merged: v128 = v128_bitselect(shifted, dup, mask);

    let indices = v128_and(merged, u8x16_splat(0x0F));

    const TABLE: v128 = u8x16(
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e',
        b'f',
    );

    let vhex: v128 = i8x16_swizzle(TABLE, indices);
    let uhex: u128 = v2u_fast(vhex);

    uhex.to_be_bytes()
}

pub enum Input {
    Partial([u8; 4096], usize),
    Page([u8; 4096]),
    Continue,
}

pub fn reader2inputs<R>(mut rdr: R) -> impl Iterator<Item = Result<Input, io::Error>>
where
    R: BufRead,
{
    let mut page: [u8; 4096] = [0; 4096];
    std::iter::from_fn(move || {
        let rslt: Result<usize, _> = rdr.read(&mut page);
        match rslt {
            Ok(0) => None,
            Ok(4096) => Some(Ok(Input::Page(page))),
            Err(e) => match e.kind() {
                ErrorKind::Interrupted => Some(Ok(Input::Continue)),
                _ => Some(Err(e)),
            },
            Ok(i) => Some(Ok(Input::Partial(page, i))),
        }
    })
}

#[target_feature(enable = "simd128,relaxed-simd")]
pub fn inputs2hex2writer<I, W>(inputs: I, mut wtr: W) -> Result<(), io::Error>
where
    I: Iterator<Item = Result<Input, io::Error>>,
    W: Write,
{
    let mut ch: [u8; 8] = [0; 8];
    for rinput in inputs {
        let input: Input = rinput?;
        match input {
            Input::Page(page) => {
                let chunks = page.chunks_exact(8);
                for chnk in chunks {
                    ch.copy_from_slice(chnk);
                    let hex: [u8; 16] = bytes2hex(ch);
                    wtr.write_all(&hex)?;
                }
            }
            Input::Partial(p, sz) => {
                let s: &[u8] = &p[..sz];
                for u in s {
                    write!(&mut wtr, "{:02x}", u)?;
                }
            }
            Input::Continue => continue,
        }
    }
    wtr.flush()
}

#[target_feature(enable = "simd128,relaxed-simd")]
pub fn stdin2hex2stdout() -> Result<(), io::Error> {
    let inputs = reader2inputs(io::stdin().lock());
    let o = io::stdout();
    let mut ol = o.lock();
    inputs2hex2writer(inputs, BufWriter::new(&mut ol))?;
    ol.flush()
}
