use wordbreaker::Dictionary;

fn main() {
    let dictionary = include_str!("../american-english-dictionary.txt")
        .lines()
        .collect::<Dictionary<_>>();

    let phrase = dictionary
        .concatenations_for("thequickbrownfoxjumpsoverthelazydog")
        .nth(71257)
        .unwrap()
        .join(" ");

    println!("{}", phrase);
}
