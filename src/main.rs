use std::io;

use rs_bytes2hex::wasm32::stdin2hex2stdout;

#[target_feature(enable = "simd128,relaxed-simd")]
fn main() -> Result<(), io::Error> {
    stdin2hex2stdout()
}
