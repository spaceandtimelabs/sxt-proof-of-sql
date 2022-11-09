/// Check that given indexes are both increasing and within range.
pub fn are_indexes_valid(ix: &[u64], n: usize) -> bool {
    let n = n as u64;
    if ix.is_empty() {
        return true;
    }
    let index = ix[0];
    if index >= n {
        return false;
    }
    let mut prev_index = index;
    for index in ix.iter().skip(1) {
        if *index <= prev_index || *index >= n {
            return false;
        }
        prev_index = *index;
    }
    true
}
