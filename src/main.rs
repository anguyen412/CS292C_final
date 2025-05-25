#[cfg(crux)] extern crate crucible;
#[cfg(crux)] use crucible::*;
use openvm::io::{read, reveal_u32};

#[cfg_attr(crux, crux::test)]
fn main() {
    let n: u64 = read();

    // Reveal the input to see the counterexample
    reveal_u32(n as u32, 100);
    reveal_u32((n >> 32) as u32, 101);
    
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..=n {                // <=  ← error
        let c = a.wrapping_add(b);
        a = b;  b = c;
    }
    reveal_u32(a as u32, 0);
    reveal_u32((a >> 32) as u32, 1);
}
