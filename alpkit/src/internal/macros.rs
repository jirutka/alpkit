macro_rules! bail {
    ($err:expr) => {
        return Err($err)
    };
}
pub(crate) use bail;
