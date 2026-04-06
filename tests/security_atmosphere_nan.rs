#[cfg(test)]
mod tests {
    use scale::layer1::atmosphere::AtmosphereGrid;
    use bevy::utils::HashMap;

    #[test]
    fn test_atmosphere_inf_propagation() {
        let mut grid = AtmosphereGrid::new(10, 10);

        // Attempt to inject Infinity
        grid.set(5, 5, f32::INFINITY);

        // Verify state
        let val = grid.get(5, 5);
        assert!(
            val.is_finite(),
            "Pollution value should be finite, found {}",
            val
        );

        // Run diffusion
        grid.diffuse(&HashMap::new(), 1.0);

        // Check if Inf propagated
        let neighbor = grid.get(5, 6);
        assert!(
            neighbor.is_finite(),
            "Neighbor pollution should be finite, found {}",
            neighbor
        );
    }
}
