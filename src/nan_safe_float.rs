use ordered_float::NotNan;
pub type Float = NotNan<f64>;

#[macro_export]
macro_rules! f {
    ( $x:literal ) => {
        Float::new($x).unwrap()
    };
}