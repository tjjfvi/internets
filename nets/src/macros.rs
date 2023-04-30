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

#[macro_export]
macro_rules! u64_0 {
  ($x:literal) => {
    if cfg!(target_endian = "big") {
      Word(($x as u64 >> 32) as u32)
    } else {
      Word($x as u64 as u32)
    }
  };
}

#[macro_export]
macro_rules! u64_1 {
  ($x:literal) => {
    if cfg!(target_endian = "big") {
      Word($x as u64 as u32)
    } else {
      Word(($x as u64 >> 32) as u32)
    }
  };
}
