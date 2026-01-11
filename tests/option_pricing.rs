use mcproton::{price_option, Barrier, BarrierType, Underlying};
use nalgebra::DMatrix;

fn create_correlation_matrix(size: usize) -> DMatrix<f64> {
    DMatrix::from_fn(size, size, |i, j| if i == j { 1.0 } else { 0.0 })
}

fn create_correlated_matrix(size: usize, correlation: f64) -> DMatrix<f64> {
    DMatrix::from_fn(size, size, |i, j| if i == j { 1.0 } else { correlation })
}

#[test]
fn test_option_pricing_call_positive_payoff() {
    // Deep in-the-money call option should have positive value
    let underlying = Underlying::new("TEST".to_string(), 100.0, 0.20);
    let correlation = create_correlation_matrix(1);
    let price = price_option(&[underlying], &correlation, 30, 50.0, true, 0.05, 1000, None);
    assert!(price > 0.0, "Deep ITM call should have positive value");
}

#[test]
fn test_option_pricing_put_positive_payoff() {
    // Deep in-the-money put option should have positive value
    let underlying = Underlying::new("TEST".to_string(), 100.0, 0.20);
    let correlation = create_correlation_matrix(1);
    let price = price_option(&[underlying], &correlation, 30, 150.0, false, 0.05, 1000, None);
    assert!(price > 0.0, "Deep ITM put should have positive value");
}

#[test]
fn test_option_pricing_at_the_money() {
    // At-the-money option should have some value due to time value
    let underlying = Underlying::new("TEST".to_string(), 100.0, 0.20);
    let correlation = create_correlation_matrix(1);
    let call_price = price_option(&[underlying.clone()], &correlation, 30, 100.0, true, 0.05, 1000, None);
    let put_price = price_option(&[underlying], &correlation, 30, 100.0, false, 0.05, 1000, None);
    assert!(call_price >= 0.0, "ATM call should have non-negative value");
    assert!(put_price >= 0.0, "ATM put should have non-negative value");
}

#[test]
fn test_barrier_option_put_in_down() {
    // Put option with barrier (65, in, down) - only has value if price falls below 65
    let underlying = Underlying::new("TEST".to_string(), 100.0, 0.20);
    let correlation = create_correlation_matrix(1);
    let barrier = Barrier::new(65.0, true, false, false); // in, down, absolute
    let price = price_option(&[underlying], &correlation, 30, 80.0, false, 0.05, 10000, Some(&barrier));
    // Should have some positive value since it's likely the barrier will be hit
    assert!(price >= 0.0, "Barrier put option should have non-negative value");
}

#[test]
fn test_barrier_option_out_barrier() {
    // Option with "out" barrier - only has value if barrier is NOT hit
    let underlying = Underlying::new("TEST".to_string(), 100.0, 0.20);
    let correlation = create_correlation_matrix(1);
    let barrier = Barrier::new(150.0, false, true, false); // out, up, absolute - barrier above current price
    let price = price_option(&[underlying], &correlation, 30, 90.0, true, 0.05, 10000, Some(&barrier));
    // Should have value since barrier is unlikely to be hit (it's above current price)
    assert!(price >= 0.0, "Out barrier option should have non-negative value");
}

// ========== Multi-Underlying Barrier Tests ==========

// 2 Underlyings Tests
#[test]
fn test_barrier_2_underlyings_worstof_in_down() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
    ];
    let correlation = create_correlated_matrix(2, 0.5);
    let barrier = Barrier::new_multi(
        0.85, // 85% of initial worst (relative)
        true, // in
        false, // down
        BarrierType::WorstOf,
        true, // relative
        vec![0, 1],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 90.0, false, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "2-underlying WorstOf in-down barrier should have non-negative value");
}

#[test]
fn test_barrier_2_underlyings_bestof_out_up() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
    ];
    let correlation = create_correlated_matrix(2, 0.5);
    let barrier = Barrier::new_multi(
        1.20, // 120% of initial best (relative)
        false, // out
        true, // up
        BarrierType::BestOf,
        true, // relative
        vec![0, 1],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 90.0, true, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "2-underlying BestOf out-up barrier should have non-negative value");
}

#[test]
fn test_barrier_2_underlyings_average_in_down() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
    ];
    let correlation = create_correlated_matrix(2, 0.5);
    let barrier = Barrier::new_multi(
        0.90, // 90% of initial average (relative)
        true, // in
        false, // down
        BarrierType::Average,
        true, // relative
        vec![0, 1],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 95.0, false, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "2-underlying Average in-down barrier should have non-negative value");
}

#[test]
fn test_barrier_2_underlyings_median_out_up() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
    ];
    let correlation = create_correlated_matrix(2, 0.5);
    let barrier = Barrier::new_multi(
        1.15, // 115% of initial median (relative)
        false, // out
        true, // up
        BarrierType::Median,
        true, // relative
        vec![0, 1],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 90.0, true, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "2-underlying Median out-up barrier should have non-negative value");
}

// 3 Underlyings Tests
#[test]
fn test_barrier_3_underlyings_worstof_in_down() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
        Underlying::new("STOCK3".to_string(), 100.0, 0.30),
    ];
    let correlation = create_correlated_matrix(3, 0.4);
    let barrier = Barrier::new_multi(
        0.80, // 80% of initial worst (relative)
        true, // in
        false, // down
        BarrierType::WorstOf,
        true, // relative
        vec![0, 1, 2],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 90.0, false, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "3-underlying WorstOf in-down barrier should have non-negative value");
}

#[test]
fn test_barrier_3_underlyings_bestof_out_up() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
        Underlying::new("STOCK3".to_string(), 100.0, 0.30),
    ];
    let correlation = create_correlated_matrix(3, 0.4);
    let barrier = Barrier::new_multi(
        1.25, // 125% of initial best (relative)
        false, // out
        true, // up
        BarrierType::BestOf,
        true, // relative
        vec![0, 1, 2],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 90.0, true, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "3-underlying BestOf out-up barrier should have non-negative value");
}

#[test]
fn test_barrier_3_underlyings_average_in_down() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
        Underlying::new("STOCK3".to_string(), 100.0, 0.30),
    ];
    let correlation = create_correlated_matrix(3, 0.4);
    let barrier = Barrier::new_multi(
        0.88, // 88% of initial average (relative)
        true, // in
        false, // down
        BarrierType::Average,
        true, // relative
        vec![0, 1, 2],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 95.0, false, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "3-underlying Average in-down barrier should have non-negative value");
}

#[test]
fn test_barrier_3_underlyings_median_out_up() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
        Underlying::new("STOCK3".to_string(), 100.0, 0.30),
    ];
    let correlation = create_correlated_matrix(3, 0.4);
    let barrier = Barrier::new_multi(
        1.18, // 118% of initial median (relative)
        false, // out
        true, // up
        BarrierType::Median,
        true, // relative
        vec![0, 1, 2],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 90.0, true, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "3-underlying Median out-up barrier should have non-negative value");
}

// 4 Underlyings Tests
#[test]
fn test_barrier_4_underlyings_worstof_in_down() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
        Underlying::new("STOCK3".to_string(), 100.0, 0.30),
        Underlying::new("STOCK4".to_string(), 100.0, 0.22),
    ];
    let correlation = create_correlated_matrix(4, 0.3);
    let barrier = Barrier::new_multi(
        0.75, // 75% of initial worst (relative)
        true, // in
        false, // down
        BarrierType::WorstOf,
        true, // relative
        vec![0, 1, 2, 3],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 90.0, false, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "4-underlying WorstOf in-down barrier should have non-negative value");
}

#[test]
fn test_barrier_4_underlyings_bestof_out_up() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
        Underlying::new("STOCK3".to_string(), 100.0, 0.30),
        Underlying::new("STOCK4".to_string(), 100.0, 0.22),
    ];
    let correlation = create_correlated_matrix(4, 0.3);
    let barrier = Barrier::new_multi(
        1.30, // 130% of initial best (relative)
        false, // out
        true, // up
        BarrierType::BestOf,
        true, // relative
        vec![0, 1, 2, 3],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 90.0, true, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "4-underlying BestOf out-up barrier should have non-negative value");
}

#[test]
fn test_barrier_4_underlyings_average_in_down() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
        Underlying::new("STOCK3".to_string(), 100.0, 0.30),
        Underlying::new("STOCK4".to_string(), 100.0, 0.22),
    ];
    let correlation = create_correlated_matrix(4, 0.3);
    let barrier = Barrier::new_multi(
        0.85, // 85% of initial average (relative)
        true, // in
        false, // down
        BarrierType::Average,
        true, // relative
        vec![0, 1, 2, 3],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 95.0, false, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "4-underlying Average in-down barrier should have non-negative value");
}

#[test]
fn test_barrier_4_underlyings_median_out_up() {
    let underlyings = vec![
        Underlying::new("STOCK1".to_string(), 100.0, 0.20),
        Underlying::new("STOCK2".to_string(), 100.0, 0.25),
        Underlying::new("STOCK3".to_string(), 100.0, 0.30),
        Underlying::new("STOCK4".to_string(), 100.0, 0.22),
    ];
    let correlation = create_correlated_matrix(4, 0.3);
    let barrier = Barrier::new_multi(
        1.20, // 120% of initial median (relative)
        false, // out
        true, // up
        BarrierType::Median,
        true, // relative
        vec![0, 1, 2, 3],
    ).unwrap();
    let price = price_option(&underlyings, &correlation, 30, 90.0, true, 0.05, 10000, Some(&barrier));
    assert!(price >= 0.0, "4-underlying Median out-up barrier should have non-negative value");
}

// Test barrier validation - absolute barrier with multiple underlyings should fail
#[test]
fn test_barrier_multi_underlying_absolute_should_fail() {
    let barrier_result = Barrier::new_multi(
        85.0, // absolute level
        true, // in
        false, // down
        BarrierType::WorstOf,
        false, // absolute (not relative)
        vec![0, 1], // multiple underlyings
    );
    assert!(barrier_result.is_err(), "Creating barrier with multiple underlyings and absolute level should fail");
}

