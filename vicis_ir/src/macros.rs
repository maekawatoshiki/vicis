macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            dbg!($($arg)*);
        }
    };
}
