#[macro_export]
macro_rules! sv_ok_tests {
    ($($name:ident => $path:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                let _ = $crate::common::assert_parse_ok($path);
            }
        )+
    };
}

#[macro_export]
macro_rules! sv_err_tests {
    ($($name:ident => $path:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                let _ = $crate::common::assert_parse_err($path);
            }
        )+
    };
}
