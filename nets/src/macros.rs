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

#[macro_export]
macro_rules! const_concat_array {
  ($c:ident = $a:ident + $b:ident) => {
    const $c: [$crate::Word; $a.len() + $b.len()] = {
      #[repr(packed)]
      struct Concat([$crate::Word; $a.len()], [$crate::Word; $b.len()]);
      unsafe { ::std::mem::transmute(Concat($a, $b)) }
    };
  };
}

#[macro_export]
macro_rules! const_payload_array {
  ($c:ident: $ty:ty = $val:expr) => {
    const $c: [$crate::Word; $crate::Length::of_payload::<$ty>().length_words()] = {
      const PADDING: usize =
        ($crate::Length::of_payload::<$ty>().length_bytes as usize) - ::std::mem::size_of::<$ty>();
      #[repr(packed)]
      struct Padded($ty, [u8; PADDING]);
      unsafe { std::mem::transmute(Padded($val, [0; PADDING])) }
    };
  };
  ($c:ident: $ty:ty) => {
    const $c: [$crate::Word; $crate::Length::of_payload::<$ty>().length_words()] =
      [$crate::Word::NULL; $crate::Length::of_payload::<$ty>().length_words()];
  };
}

pub use internets_interactions_macro::interactions as __interactions;

#[macro_export]
macro_rules! interactions {
  ($($x:tt)*) => {
    $crate::__interactions! {
      use $crate;
      $($x)*
    }
  };
}
