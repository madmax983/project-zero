fn main() {
    let parts = vec!["hello", " ", "world"];
    let s: String = parts.iter().map(|s| *s).collect(); // or parts.iter().copied().collect::<String>()
    println!("{}", s);
}
