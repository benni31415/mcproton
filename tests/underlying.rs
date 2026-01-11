use mcproton::Underlying;

#[test]
fn test_underlying_creation() {
    let underlying = Underlying::new("TEST".to_string(), 100.0, 0.20);
    assert_eq!(underlying.name, "TEST");
    assert_eq!(underlying.spot_price, 100.0);
    assert_eq!(underlying.volatility, 0.20);
}

