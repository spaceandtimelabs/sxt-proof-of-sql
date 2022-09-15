/**
 * Adopted from arrow
 *
 * See third_party/license/arrow.LICENSE
 */
use datafusion::arrow::array::Array;
use std::fmt;

// Helper function for printing potentially long arrays.
pub(super) fn print_long_array<A, F>(
    array: &A,
    f: &mut fmt::Formatter,
    print_item: F,
) -> fmt::Result
where
    A: Array,
    F: Fn(&A, usize, &mut fmt::Formatter) -> fmt::Result,
{
    let head = std::cmp::min(10, array.len());

    for i in 0..head {
        if array.is_null(i) {
            writeln!(f, "  null,")?;
        } else {
            write!(f, "  ")?;
            print_item(array, i, f)?;
            writeln!(f, ",")?;
        }
    }
    if array.len() > 10 {
        if array.len() > 20 {
            writeln!(f, "  ...{} elements...,", array.len() - 20)?;
        }

        let tail = std::cmp::max(head, array.len() - 10);

        for i in tail..array.len() {
            if array.is_null(i) {
                writeln!(f, "  null,")?;
            } else {
                write!(f, "  ")?;
                print_item(array, i, f)?;
                writeln!(f, ",")?;
            }
        }
    }
    Ok(())
}
