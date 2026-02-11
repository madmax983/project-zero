use scale::experimental::miasma::MiasmaGrid;

fn main() {
    let grid = MiasmaGrid::new(10, 10);
    println!("Miasma grid created with size {}x{}", grid.width, grid.height);
}
