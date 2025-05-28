#[cfg(crux)] extern crate crucible;
#[cfg(crux)] use crucible::*;

// BUG: missing .wrapping_add or range assertion ⇒ overflow mismatch
#[cfg_attr(crux, crux::test)]
fn main() {
    let x = u32::symbolic("x");
    assert!(x == x);
}

#[cfg_attr(crux, crux::test)]
fn overflow_test() {
    let x = u32::symbolic("x");
    let y = u32::symbolic("y");

    let z = x + y;

    assert!(z >= x && z >= y);
}

#[cfg_attr(crux, crux::test)]
fn no_overflow_test() {
    let x = u32::symbolic("x");
    let y = u32::symbolic("y");

    // Restrict so no overflow is possible
    crucible_assume!(x <= 2_000_000_000);
    crucible_assume!(y <= 2_000_000_000);

    let z = x + y;

    assert!(z >= x && z >= y);
}