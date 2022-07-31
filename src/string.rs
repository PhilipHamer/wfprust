pub type WideString = &'static [u16];

#[macro_export]
macro_rules! wide_string {
    ($s: expr) => { &obfstr::wide!(concat!($s, "\0")) };
}
