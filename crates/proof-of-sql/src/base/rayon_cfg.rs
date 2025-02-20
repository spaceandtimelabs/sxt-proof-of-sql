macro_rules! if_rayon {
    ($rayon_value: expr_2021, $else_value: expr_2021) => {{
        #[cfg(feature = "rayon")]
        {
            ($rayon_value)
        }
        #[cfg(not(feature = "rayon"))]
        {
            ($else_value)
        }
    }};
}
pub(crate) use if_rayon;
