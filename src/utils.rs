//! Utility functions matching Python textdistance.utils

/// Extracts n-grams from a slice of elements.
///
/// Returns a vector of vectors, where each inner vector represents an n-gram.
pub fn find_ngrams<T: Clone>(input_list: &[T], n: usize) -> Vec<Vec<T>> {
    if n == 0 || input_list.len() < n {
        return Vec::new();
    }
    input_list
        .windows(n)
        .map(|window| window.to_vec())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_ngrams() {
        let input: Vec<char> = "hello".chars().collect();
        let ngrams = find_ngrams(&input, 2);
        assert_eq!(ngrams.len(), 4);
        assert_eq!(ngrams[0], vec!['h', 'e']);
        assert_eq!(ngrams[1], vec!['e', 'l']);
        assert_eq!(ngrams[2], vec!['l', 'l']);
        assert_eq!(ngrams[3], vec!['l', 'o']);
    }
}
