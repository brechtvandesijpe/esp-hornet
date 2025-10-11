#![no_std]

#[macro_export]
macro_rules! declare_feature_tag {
    ($name:expr) => {
        #[allow(non_upper_case_globals)]
        const FEATURE_TAG: &str = $name;
    };
}

#[macro_export]
macro_rules! flog {
    // usage: flog!(info, "message {}" , arg);
    ($lvl:ident, $fmt:expr $(, $args:expr )* $(,)?) => {
        ::log::$lvl!(concat!("[{}] ", $fmt) , FEATURE_TAG $(, $args)*)
    };
}