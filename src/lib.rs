use fst::raw::{Fst, Node};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

pub fn load_dictionary<T>(words: &[T]) -> fst::Result<Fst<Vec<u8>>>
where
    T: AsRef<str>,
{
    let mut words = words
        .iter()
        .map(|word| word.as_ref())
        .filter(|word| !word.is_empty())
        .map(|word| word.chars().nfd().collect::<String>())
        .collect::<Vec<_>>();
    words.sort_unstable_by(|word1, word2| word1.as_bytes().cmp(word2.as_bytes()));
    words.dedup();

    Fst::from_iter_set(words.iter())
}

pub fn break_into_words<D>(word: &str, dictionary: &Fst<D>) -> Vec<Vec<String>>
where
    D: AsRef<[u8]>,
{
    fn _break_into_words<D, S>(
        results: &mut Vec<Vec<String>>,
        prefix: &mut Vec<String>,
        graphemes: &[S],
        dictionary: &Fst<D>,
        root: Node<'_>,
    ) where
        D: AsRef<[u8]>,
        S: AsRef<str>,
    {
        prefix.push(String::new());
        let mut current_node = root;

        'outer: for (i, grapheme) in graphemes
            .iter()
            .map(|grapheme| grapheme.as_ref())
            .enumerate()
        {
            prefix.last_mut().unwrap().push_str(grapheme);

            for byte in grapheme.bytes() {
                let transition_index = match current_node.find_input(byte) {
                    Some(index) => index,
                    None => break 'outer,
                };
                current_node = dictionary.node(current_node.transition(transition_index).addr);
            }

            if current_node.is_final() {
                if i == graphemes.len() - 1 {
                    results.push(prefix.clone());
                } else {
                    _break_into_words(results, prefix, &graphemes[i + 1..], dictionary, root);
                }
            }
        }

        prefix.pop();
    }

    if word.is_empty() {
        return vec![Vec::new()];
    }

    let mut results = Vec::new();
    let mut prefix = Vec::new();
    let word = word.chars().nfd().collect::<String>();
    let graphemes = word.graphemes(true).collect::<Vec<_>>();
    let root = dictionary.root();

    _break_into_words(&mut results, &mut prefix, &graphemes, dictionary, root);

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let dictionary = load_dictionary(&["b"]).unwrap();
        let ways_to_concatenate = break_into_words("a", &dictionary);

        assert!(ways_to_concatenate.is_empty());
    }

    #[test]
    fn test_2() {
        let dictionary = load_dictionary(&["b"]).unwrap();
        let ways_to_concatenate = break_into_words("", &dictionary);

        assert!(ways_to_concatenate.len() == 1);
        assert!(ways_to_concatenate[0].is_empty());
    }

    #[test]
    fn test_3() {
        let dictionary = load_dictionary(&["ab", "abc", "cd", "def", "abcd", "ef", "c"]).unwrap();
        let mut ways_to_concatenate = break_into_words("abcdef", &dictionary);
        ways_to_concatenate.sort_unstable();

        assert_eq!(
            ways_to_concatenate,
            [
                vec!["ab", "c", "def"],
                vec!["ab", "cd", "ef"],
                vec!["abc", "def"],
                vec!["abcd", "ef"]
            ]
        );
    }
}
