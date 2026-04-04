use ratatui::buffer::Cell;

fn main() {
    let mut cell = Cell::default();
    cell.set_symbol("X");
    let content = vec![cell];

    let old_way: String = content.iter().map(|c| c.symbol()).collect::<Vec<_>>().join("");
    let new_way: String = content.iter().map(|c| c.symbol()).collect::<String>();

    assert_eq!(old_way, new_way);
    println!("Works! {}", new_way);
}
