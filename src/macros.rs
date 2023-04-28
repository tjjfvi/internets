#[macro_export]
macro_rules! fail {
  ($x:expr) => {
    if cfg!(feature = "unsafe") {
      unsafe { std::hint::unreachable_unchecked() }
    } else {
      $x
    }
  };
}

#[macro_export]
macro_rules! safe {
  ($($x:tt)*) => {
    if cfg!(not(feature = "unsafe")) {
      $($x)*
    }
  };
}
