fn main() {
    let parts = vec!["hello", " ", "world"];
    let s: String = parts.iter().copied().collect();
    println!("{}", s);
}
