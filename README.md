# pack4x5

Last week, I read an interesting trick for storing 4 5-bit numbers into 16 bits: https://github.com/isometric/BucketCompressionTrick

It works by dropping the ordering of the numbers, which reduces the number of possible combinations. Four 4-bit numbers would normally fill `2**16 = 65536` combinations, but if order doesn't matter, there are only `multichoose(16, 4) = 3876` combinations, which would fit in 12 bits (`2**12 = 4096`).

I thought I would try to improve two things about the [sample code](https://github.com/isometric/BucketCompressionTrick/blob/master/main.cpp):

- It uses a loop to iterate over all 3876 possible combinations, and builds a mapping table in each direction. These tables will be 8KB and 128KB respectively. Even though they make the compression fast, it occurred to me that for some applications of this kind of compression, 128KB might not be easy to come by.

- Because the lookup tables are so large, it only compresses 4 bits of each number this way. The lowest bit of each number is stripped off and filled in at the end. (Effectively it compresses 4 4-bit numbers into 12 bits, and uses the other 4 bits as normal storage.) This adds complexity that you could avoid if you compress all 5 bits at once.

Also, because I love a challenge, I decided to try writing it in rust instead of C++.


## Math

For 4 5-bit numbers, there are `multichoose(32, 4) = 52360` possible combinations that ignore sort order, and that will fit perfectly into 16 bits.

To compress and decompress without a table, I assume that the numbers have been reverse-sorted (same as the original code) with the largest number first. Then I assume that there's an imaginary list of all the possible combinations, sorted from smallest (0 0 0 0, then 1 0 0 0, then 1 1 0 0...) to largest (31 31 31 31).

If the first number is 2, then its sort order is above every possible combination with a 0 or 1 as the first number, so its sort order is at _least_ `multichoose(2, 4) = 5`. And it must be less than any combination starting with 3 (`multichoose(3, 4) = 15`).

Given that the first number is 2, the number of possible combinations for the remaining digits is `multichoose(2, 3) = 10`. So you can recursively narrow the range by counting, for each number, the combinations that would be ordered below it.

```
[ 14, 12, 12, 4 ] = multichoose(14, 4) + multichoose(12, 3) + multichoose(12, 2) + multichoose(4, 1)
    = 2380 + 364 + 78 + 4
    = 2826
```

Decompressing works the opposite way: Use a 5-stage binary search to find the lower bound in `multichoose(n, 4)`, and `n` will be the first number. Subtract that bound and do the same for each number.

These steps are simple, but they rely on a fairly efficient way to calculate the multichoose function.


## Efficient small multichoose

The formula for multichoose is

```
                                            (n + r - 1)!
multichoose(n, r) = choose(n + r - 1, r) = --------------
                                             r! (n - 1)!
```

`r` will always be between 1 and 4 inclusive, so `r!` will always be pretty small too (1, 2, 6, or 24). The other two terms amount to "the product of `r` consecutive integers starting at `n`", so the small `r` helps there too.

To make it relatively efficient, I turned the multichoose function into a match statement for the 4 possible values of `r`, using shifts to divide by 2 and 4.

To divide by 3, I used a multiplication trick. On tiny devices, division is often implemented as a (slow) library call, but if you're dividing by a constant, you can usually find a good equivalent fixed-point constant. "One third" in binary is "0.5555..." repeating, so using a 16-bit fixed-point fraction, you can multiply by 0x5556 and shift right 16 bits. I can keep it down to 16 bits because I know the value of `n` will never be higher than 32. (Compilers are smart enough to do this trick without hand-holding, but it was fun to keep the intermediate results down to 16 bits.)

I also "know" the result of the division will have no remainder. Any two consecutive integers will have one that is even (divisible by two), so the product will be even also. The same rule holds for 3 consecutive integers being divisible by three, and so on.


## It works

The code is pretty small: less than 100 lines if you don't include a test suite that is somewhat overkill. I also didn't fight with the rust compiler as much as I feared. It was mostly concerned that I mark the various `u16` type conversions very explicitly.

I'm not sure this packing trick is particularly useful in the real world, but maybe someday you'll need it.
