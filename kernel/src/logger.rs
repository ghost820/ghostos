#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {{
        $crate::serial_println!("[debug] {}", format_args!($($arg)+));
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {{
        $crate::serial_println!("[info] {}", format_args!($($arg)+));
    }};
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)+) => {{
        $crate::serial_println!("[warning] {}", format_args!($($arg)+));

        #[cfg(debug_assertions)]
        panic!("warning log triggered");
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {{
        $crate::serial_println!("[error] {}", format_args!($($arg)+));

        #[cfg(debug_assertions)]
        panic!("error log triggered");
    }};
}

#[macro_export]
macro_rules! critical {
    ($($arg:tt)+) => {{
        $crate::serial_println!("[critical] {}", format_args!($($arg)+));
        panic!("critical log triggered");
    }};
}
