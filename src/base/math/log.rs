use std::mem;

pub fn log2_down(x: usize) -> usize {
    mem::size_of::<usize>() * 8 - (x.leading_zeros() as usize) - 1
}

pub fn log2_up(x: usize) -> usize {
    let is_not_pow_2 = (x & (x - 1) != 0) as usize;
    log2_down(x) + is_not_pow_2
}
