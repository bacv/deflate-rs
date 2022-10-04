fn next_pow2(n: usize) -> usize {
    let mut n = n - 1;
    let mut i = 0;

    while i <= 4 {
        n |= n >> 2u8.pow(i);
        i += 1;
    }

    n + 1
}

fn to_code(length: usize, distance: usize) -> Vec<u8> {
    let mut ld = short_be_bytes(length);
    ld.append(&mut ":".as_bytes().to_vec());
    ld.append(&mut short_be_bytes(distance));

    ld
}

fn short_be_bytes(d: usize) -> Vec<u8> {
    let mut np2 = next_pow2(d);
    let mut exp = 1; // exponent

    // The representation will always take at least one byte.
    if np2 < 2 {
        np2 = 2;
    }

    // Get the pow of a number.
    // This counts the number of bits required to represent a number up to the least significant
    // bit, but does not include it:
    // 8 4 2 1
    // 1 1 1 0
    // The code accounts for the first bit in the line where `pow` is divided by 8.
    while np2 != 2 {
        np2 /= 2;
        exp += 1;
    }

    // This might overflow and panic.
    // Adding one to `pow` to account for a first bit.
    exp = (exp + 1 + 8 - 1) / 8;

    let mut res = vec![0u8; exp];

    // Inserting the least significant byte to the front of res.
    for (i, b) in d.to_le_bytes().iter().enumerate() {
        if i > exp - 1 {
            break;
        }
        res[i] = *b;
    }

    // Reversing res to have most significant bytes at the front.
    res.reverse();
    res
}

/// This function returns length of a match and the distance from it's possition in the search
/// cursor. The last param return in the Option represents where the method stopped. It's useful to
/// start from this position if all the `data` provided to the method matched and it's unknown if
/// the next element will be a match as well.
fn find_repeat_element(
    search_buf: &[u8],
    data: &[u8],
    data_offset: usize,
    search_offset: usize,
) -> Option<(usize, usize, usize)> {
    let mut coords = None;
    let mut length = 0;
    let mut last_idx = 0;
    let mut is_prev_match = false;

    // Start at the beginning of the data that has been read already.
    for (i, sb) in search_buf[search_offset..].iter().enumerate() {
        // Increase the index to calculate the distance later.
        last_idx = i;

        // If our current buffer that we want to compress is traversed then break.
        if data.len() <= length {
            break;
        }

        // if we have a sequence of matches, increase the length of data that will be compressed.
        if data[length] == *sb {
            is_prev_match = true;
            length += 1;
            continue;
        }

        if is_prev_match {
            break;
        }

        length = 0;
    }

    // Check if we have anything to compress and if we are not looking into the search buffer
    // further than the given offset.
    if length != 0 && data_offset + length > last_idx {
        let dist = data_offset + length - last_idx;
        coords = Some((length, dist, last_idx));
    }

    coords
}

fn find_repeat_elements(data: &[u8]) -> Vec<u8> {
    // TODO: make this configurable.
    let offset = 5usize;

    // Search buffer should have the data skipped by the for loop.
    let mut search_buf = data[..offset].to_vec();
    let mut current_buf = Vec::default();
    let mut out = search_buf.clone();

    // A cursor to track the where previous match ended in search buf.
    let mut search_cursor = 0usize;

    // Track if previous iteration was a match and try to compare only the next element after the
    // `search_cursor`.
    let mut is_prev_match = false;

    // Start searching for matches a bit further away.
    for (i, b) in data[offset..].iter().enumerate() {
        let offset = i + offset;
        // Append every char into search buffer.
        search_buf.push(*b);
        current_buf.push(*b);

        // Iterate over elements in the `current buffer to compress`.
        if let Some((l, d, idx)) =
            find_repeat_element(&search_buf, &current_buf, offset, search_cursor)
        {
            // TODO: come up with a proper rules what should be compressed and what should be
            // skipped.
            if l > 3 && d > 3 {
                out.append(&mut to_code(l, d));
                current_buf.clear();
            }

            if is_prev_match {
                search_cursor = idx;
            }

            is_prev_match = true;
        } else {
            search_cursor = 0;
            is_prev_match = false;
            out.push(search_buf[offset]);
        };
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_code() {
        let (length, distance) = (256, 20);
        let coord = to_code(length, distance);

        // `256:20`.
        assert_eq!(coord, &[1, 0, 58, 20]);
        assert_eq!(to_code(1, 1), &[1, 58, 1]);
    }

    #[test]
    fn test_find_first_duplicates_length_distance() {
        let data = [10, 20, 30, 40, 10, 20, 30, 40, 50];
        let buf = [10, 20, 30, 40];
        let coords = find_repeat_element(&data, &buf, 4, 0);
        assert_eq!(coords, Some((4, 4, 4)));

        let data = [10, 20, 20, 40, 10, 20, 30, 40, 50]; // 10, 20, 30, 40
        let buf = [10, 20, 30, 40];
        // only first two elements from `buf` should be returned as a match.
        let coords = find_repeat_element(&data, &buf, 9, 0);
        assert_eq!(coords, Some((2, 9, 2)));

        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; // 7, 8, 9
        let buf = [7, 8, 9];
        let coords = find_repeat_element(&data, &buf, 10, 0);
        // assuming `buf` is at the end of `data`
        assert_eq!(coords, Some((3, 4, 9)));

        let data = [1, 2, 3, 4, 5, 6, 7, 9, 9, 10];
        let buf = [7, 8, 9];
        let coords = find_repeat_element(&data, &buf, 10, 0);
        assert_eq!(coords, Some((1, 4, 7)));
    }

    #[test]
    fn test_replace_duplicates_w_lenght_distance() {
        let data = [10, 20, 30, 40, 50, 10, 20, 30, 40, 60];
        let expected = [10, 20, 30, 40, 50, 4, 58, 4, 60];
        let compressed = find_repeat_elements(&data);

        assert_eq!(compressed, expected);
    }
}
