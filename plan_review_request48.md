Wait, `minimal_nova_demo.rs` also uses the same table! If I implement `Display` for `OralTradition`, I should ALSO simplify `examples/minimal_nova_demo.rs` to just use `println!("{}", tradition);` instead of repeating the table logic!

Let's check `examples/minimal_nova_demo.rs`.
