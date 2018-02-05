const ONE_THIRD_15Q16: u32 = 0x5556;

#[inline]
fn div3(n: u32) -> u32 {
  ((n * ONE_THIRD_15Q16) >> 16)
}

/// `multichoose(n, r) = choose(n + r - 1, r) = (n + r - 1)! / (r!(n - 1)!)`.
///
/// `r` will be in `1..5`. `n` will be in `0..33`. because of this, we
/// can hardcode `r!` with shifts and a division-by-multiplying (for 3).
/// the other two factors cancel to be `product(n .. n + r)`.
///
/// for example, `multichoose(16, 3)` is really `16 * 17 * 18` divided by
/// 2, divided by 3. (having two sequential numbers as factors guarantees
/// that at least one of them will be a multiple of 2, so the product will
/// divide evenly. the same logic holds for 3 and on.)
fn multichoose(n: u8, r: u8) -> u16 {
  debug_assert!(r > 0 && r <= 4);
  debug_assert!(n <= 32);

  match n as u32 {
    0 => 0,
    1 => 1,
    2 => (r + 1) as u16,
    3 => (((r + 1) * (r + 2)) >> 1) as u16,
    n => (match r {
      1 => n,
      2 => (n * (n + 1)) >> 1,
      3 => div3(((n * (n + 1)) >> 1) * (n + 2)),
      4 => (div3(((n * (n + 1)) >> 1) * (n + 2)) * (n + 3)) >> 2,
      // can't happen -- just satisfies the compiler:
      _ => 0
    }) as u16
  }
}

/// Pack 4 5-bit numbers into a u16. The numbers are preserved, but their
/// order isn't. Decode with `unpack4x5`.
pub fn pack4x5(list: &mut [u8; 4]) -> u16 {
  debug_assert!(list[0] < 32 && list[1] < 32 && list[2] < 32 && list[3] < 32);
  list.sort_by(|a, b| b.cmp(a));
  multichoose(list[0], 4) + multichoose(list[1], 3) + multichoose(list[2], 2) + (list[3] as u16)
}

#[inline]
fn binary_unchoose(sum: &mut u16, r: u8) -> u8 {
  let mut lo = 0;
  let mut hi = 32;
  let mut step = 16;
  let mut floor = 0;

  loop {
    if lo + 1 == hi {
      *sum -= floor;
      return lo;
    }
    let level = multichoose(lo + step, r);
    if *sum >= level {
      lo += step;
      floor = level;
    } else {
      hi -= step;
    }
    step >>= 1;
  }
}

/// Unpack an array of 4 5-bit numbers that were previously packed with
/// `pack4x5`. The numbers are preserved, but their order isn't.
pub fn unpack4x5(mut packed: u16) -> [u8; 4] {
  [
    binary_unchoose(&mut packed, 4),
    binary_unchoose(&mut packed, 3),
    binary_unchoose(&mut packed, 2),
    packed as u8
  ]
}


// ----- tests

#[cfg(test)]
mod tests {
  use multichoose;
  use pack4x5;
  use unpack4x5;

  #[test]
  fn test_multichoose() {
    assert_eq!(multichoose(1, 1), 1);
    assert_eq!(multichoose(16, 1), 16);
    assert_eq!(multichoose(1, 2), 1);
    assert_eq!(multichoose(3, 2), 6);
    assert_eq!(multichoose(5, 2), 15);
    assert_eq!(multichoose(16, 2), 136);
    assert_eq!(multichoose(32, 2), 528);
    assert_eq!(multichoose(1, 3), 1);
    assert_eq!(multichoose(2, 3), 4);
    assert_eq!(multichoose(13, 3), 455);
    assert_eq!(multichoose(29, 3), 4495);
    assert_eq!(multichoose(32, 3), 5984);
    assert_eq!(multichoose(1, 4), 1);
    assert_eq!(multichoose(2, 4), 5);
    assert_eq!(multichoose(17, 4), 4845);
    assert_eq!(multichoose(29, 4), 35960);
    assert_eq!(multichoose(32, 4), 52360);
  }

  #[test]
  fn test_pack4x5() {
    assert_eq!(pack4x5(&mut [ 0, 0, 0, 0 ]), 0);
    assert_eq!(pack4x5(&mut [ 1, 0, 0, 0 ]), 1);
    assert_eq!(pack4x5(&mut [ 0, 0, 1, 0 ]), 1);
    assert_eq!(pack4x5(&mut [ 1, 0, 1, 0 ]), 2);
    assert_eq!(pack4x5(&mut [ 1, 1, 1, 1 ]), 4);
    assert_eq!(pack4x5(&mut [ 2, 0, 0, 0 ]), 5);
    assert_eq!(pack4x5(&mut [ 8, 0, 0, 0 ]), 330);
    assert_eq!(pack4x5(&mut [ 8, 5, 4, 3 ]), 378);
    assert_eq!(pack4x5(&mut [ 8, 8, 8, 8 ]), 494);
    assert_eq!(pack4x5(&mut [ 14, 12, 12, 4 ]), 2826);
    assert_eq!(pack4x5(&mut [ 31, 0, 0, 0 ]), 46376);
    assert_eq!(pack4x5(&mut [ 11, 22, 9, 30 ]), 43019);
  }

  #[test]
  fn test_unpack4x5() {
    assert_eq!(unpack4x5(0), [ 0, 0, 0, 0 ]);
    assert_eq!(unpack4x5(1), [ 1, 0, 0, 0 ]);
    assert_eq!(unpack4x5(2), [ 1, 1, 0, 0 ]);
    assert_eq!(unpack4x5(4), [ 1, 1, 1, 1 ]);
    assert_eq!(unpack4x5(5), [ 2, 0, 0, 0 ]);
    assert_eq!(unpack4x5(330), [ 8, 0, 0, 0 ]);
    assert_eq!(unpack4x5(378), [ 8, 5, 4, 3 ]);
    assert_eq!(unpack4x5(494), [ 8, 8, 8, 8 ]);
    assert_eq!(unpack4x5(2826), [ 14, 12, 12, 4 ]);
    assert_eq!(unpack4x5(46376), [ 31, 0, 0, 0 ]);
    assert_eq!(unpack4x5(43019), [ 30, 22, 11, 9 ]);
  }
}
