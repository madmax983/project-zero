fn main() {
    let parts = vec!["h", "e", "l", "l", "o"];
    let s: String = parts.iter().map(|&s| s).collect();
    println!("{}", s);
}
