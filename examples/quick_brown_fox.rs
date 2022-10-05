use wordbreaker::Dictionary;

fn main() {
    let dictionary = include_str!("../american-english-dictionary.txt")
        .lines()
        .collect::<Dictionary<_>>();

    let phrase = dictionary
        .word_segmentations("thequickbrownfoxjumpsoverthelazydog")
        .nth(71257)
        .unwrap()
        .join(" ");

    println!("{}", phrase);
}
