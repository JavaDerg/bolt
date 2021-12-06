use smallvec::SmallVec;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn find_slashes(url: &str, delim: Option<usize>) -> SmallVec<[usize; 8]> {
    if is_x86_feature_detected!("sse2")
        && is_x86_feature_detected!("bmi2")
        && is_x86_feature_detected!("popcnt")
    {
        unsafe { find_slashes_sse2(url.as_bytes(), delim.unwrap_or(usize::MAX)) }
    } else {
        find_slashes_fallback(
            url.as_bytes(),
            0,
            delim.unwrap_or(usize::MAX),
            SmallVec::new(),
        )
    }
}

/// This function is unsafe due to simd operations
/// This functions requires sse2, bmi2 & popcnt
///
/// SAFETY: length of string is verified, only 16 byte blocks are read, fallback mode for >16 pieces
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
#[target_feature(enable = "bmi2")]
#[target_feature(enable = "popcnt")]
unsafe fn find_slashes_sse2(url: &[u8], delim: usize) -> SmallVec<[usize; 8]> {
    let mut vec = SmallVec::new();

    let byte = delim % 16;
    let key = delim - byte;
    let mut temp = [0u8; 16];

    let url_ptr = url.as_ptr();

    let max_mask = rshiftb(_mm_set1_epi8(-1), byte);
    let slash = _mm_set1_epi8(b'/' as i8);

    for i in (0..url.len().min(delim)).step_by(16) {
        let left = url.len() - i;
        let mut invec = if left >= 16 {
            _mm_loadu_si128(url_ptr.add(i) as *const __m128i)
        } else {
            // SAFETY: temp and url can not overlap as temp lives in the local stack, while url is a parameter from the outside
            //         The max length is checked above, memory out side of the boundaries is not read
            std::ptr::copy_nonoverlapping(url.as_ptr().add(i), temp.as_mut_ptr(), left);
            std::ptr::copy_nonoverlapping(
                [0u8; 16].as_ptr(),
                temp.as_mut_ptr().add(left),
                16 - left,
            );
            _mm_loadu_si128(temp.as_ptr() as *const __m128i)
        };

        if i == key {
            invec = _mm_andnot_si128(max_mask, invec);
        }
        let mask = _mm_cmpeq_epi8(invec, slash);
        let mask = _mm_movemask_epi8(mask) as u64;

        // https://stackoverflow.com/a/45513660
        // Full credit to Aki Suihkonen for this incredible bit-hackery

        // spreads mask 1bit -> 4bit & fills up 4 bits with 1 bit value
        // see: https://www.felixcloutier.com/x86/pdep
        let mask4b = _pdep_u64(mask, 0x1111_1111_1111_1111) * 0xF;
        // selects relevant indexes and compresses them
        // see https://www.felixcloutier.com/x86/pext
        let indices = _pext_u64(0xfedcba9876543210, mask4b);

        // spreads 4 bit values out into 8 bit values, only taking half of it
        // so we take 2 and join them as 128 bit integer
        let high = _pdep_u64(indices >> 32, 0x0F0F_0F0F_0F0F_0F0F);
        let low = _pdep_u64(indices, 0x0F0F_0F0F_0F0F_0F0F);
        let indices = _mm_loadu_si128([low, high].as_ptr() as *const __m128i);

        // count how many matches we've got
        let len = _popcnt64(mask as i64) as usize;

        let mut array = [0u8; 16];
        _mm_storeu_si128(array.as_mut_ptr() as *mut __m128i, indices);
        (&array[..len])
            .iter()
            .map(|x| i + *x as usize)
            .for_each(|x| vec.push(x));
    }

    vec
}

fn find_slashes_fallback(
    url: &[u8],
    start: usize,
    delim: usize,
    mut vec: SmallVec<[usize; 8]>,
) -> SmallVec<[usize; 8]> {
    (start..url.len().min(delim))
        .filter(|i| url[*i] == b'/')
        .for_each(|i| vec.push(i));
    vec
}

macro_rules! rshift {
    ($val:expr; $shift:expr; $($num:literal) +) => {
        match $shift {
            $(
                $num => _mm_bsrli_si128::<$num>($val),
            )+
            _ => panic!("not covered"),
        }
    };
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn rshiftb(a: __m128i, shift: usize) -> __m128i {
    rshift!(a; shift; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15)
}
