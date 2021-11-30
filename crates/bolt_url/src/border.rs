fn find_border(url: &str) -> (Option<usize>, Option<usize>) {
    if is_x86_feature_detected!("sse2") {
        unsafe { find_border_sse2(url.as_bytes()) }
    } else {
        find_border_fallback(url.as_bytes(), 0)
    }
}

/// This function is unsafe due to simd operations
///
/// SAFETY: length of string is verified, only 16 byte blocks are read, fallback mode for >16 pieces
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn find_border_sse2(url: &[u8]) -> (Option<usize>, Option<usize>) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut last = 0;
    let mut query_loc = None;

    if url.len() - last > 16 {
        let hash = _mm_set1_epi8(b'#' as i8);
        let query = _mm_set1_epi8(b'?' as i8);

        let url_ptr = url.as_ptr();

        for i in (last..url.len()).step_by(16) {
            last = i;
            let invec = _mm_loadu_si128(url_ptr.offset(i as isize) as *const __m128i);

            let mask_query = _mm_cmpeq_epi8(invec, query);
            let mask_i32_query = _mm_movemask_epi8(mask_query);

            let mask_hash = _mm_cmpeq_epi8(invec, hash);
            let mask_i32_hash = _mm_movemask_epi8(mask_hash);

            if mask_i32_query != 0 && query_loc.is_none() {
                query_loc = Some(i + mask_i32_query.trailing_zeros() as usize);
            }

            if mask_i32_hash != 0 {
                return (query_loc, Some(i + mask_i32_hash.trailing_zeros() as usize));
            }
        }
    }

    find_border_fallback(url, last)
}

fn find_border_fallback(url: &[u8], start: usize) -> (Option<usize>, Option<usize>) {
    let mut query_loc = None;
    for i in start..url.len() {
        if query_loc.is_none() && url[i] == b'?' {
            query_loc = Some(i);
            continue;
        }
        if url[i] == b'#' {
            return (query_loc, Some(i));
        }
    }
    (query_loc, None)
}
