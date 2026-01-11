use mcproton::{price_option, Underlying};
use nalgebra::DMatrix;

fn main() {
    // Example usage
    let underlying = Underlying::new(
        "AAPL".to_string(),
        150.0,  // Current spot price
        0.25,   // 25% annual volatility
    );
    
    // Create correlation matrix (1x1 identity matrix for single underlying)
    let correlation_matrix = DMatrix::from_row_slice(1, 1, &[1.0]);
    
    let time_horizon_days = 30;
    let strike_price = 155.0;
    let risk_free_rate = 0.05; // 5% annual risk-free rate
    let num_paths = 1000;
    
    // Price a Call option (vanilla, no barrier)
    let call_price = price_option(
        &[underlying.clone()],
        &correlation_matrix,
        time_horizon_days,
        strike_price,
        true,  // is_call = true
        risk_free_rate,
        num_paths,
        None,  // No barrier
    );
    
    // Price a Put option (vanilla, no barrier)
    let put_price = price_option(
        &[underlying.clone()],
        &correlation_matrix,
        time_horizon_days,
        strike_price,
        false,  // is_call = false
        risk_free_rate,
        num_paths,
        None,  // No barrier
    );
    
    println!("Monte Carlo Option Pricing");
    println!("==========================");
    println!("Underlying: {}", underlying.name);
    println!("Spot Price: ${:.2}", underlying.spot_price);
    println!("Volatility: {:.2}%", underlying.volatility * 100.0);
    println!("Time Horizon: {} days", time_horizon_days);
    println!("Strike Price: ${:.2}", strike_price);
    println!("Risk-Free Rate: {:.2}%", risk_free_rate * 100.0);
    println!("Number of Paths: {}", num_paths);
    println!("==========================");
    println!("Estimated CALL Option Price: ${:.4}", call_price);
    println!("Estimated PUT Option Price: ${:.4}", put_price);
}

