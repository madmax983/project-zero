#[cfg(test)]
mod tests {
    use scale::layer1::crowding::CrowdingGrid;

    #[test]
    #[should_panic(expected = "Grid size overflow or too large")]
    fn test_crowding_grid_overflow_dos() {
        // Attempt to create a grid where width * height overflows usize.
        // On 64-bit, usize::MAX is 2^64 - 1.
        // Let's use width = 2^32 + 1, height = 2^32 + 1. Product overflows.
        // But we can't allocate 2^64 bytes.

        // We need a case where width * height wraps to a small number,
        // but width and height are seemingly valid large numbers.

        // Example: width = usize::MAX / 2 + 2. height = 2.
        // width * height = (MAX/2 + 2) * 2 = MAX + 4. Wraps to 3 (or similar small number).

        let width = (usize::MAX / 2) + 2;
        let height = 2;

        // This should allocate a small vector due to overflow wrapping
        let mut grid = CrowdingGrid::new(width, height);

        // This access is "valid" by the bounds check (y < height, x < width)
        // but the index will be large, and the vector is small.
        // It should panic.
        grid.add_crowding(0, 1, 10);
    }
}
