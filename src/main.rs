mod underlying;

use rand_distr::{Distribution, Normal};
use underlying::Underlying;

/// Prices a European option (Call or Put) using Monte Carlo simulation
///
/// # Arguments
/// * `underlying` - The underlying asset
/// * `time_horizon_days` - Time to expiration in days
/// * `strike_price` - Strike price of the option
/// * `is_call` - `true` for Call option, `false` for Put option
/// * `risk_free_rate` - Annual risk-free interest rate (as a decimal, e.g., 0.05 for 5%)
/// * `num_paths` - Number of Monte Carlo simulation paths
///
/// # Returns
/// The estimated option price
pub fn price_option(
    underlying: &Underlying,
    time_horizon_days: u32,
    strike_price: f64,
    is_call: bool,
    risk_free_rate: f64,
    num_paths: usize,
) -> f64 {
    let time_to_expiration = time_horizon_days as f64 / 365.0; // Convert days to years
    let dt = time_to_expiration; // Single step to expiration
    let sqrt_dt = dt.sqrt();
    
    // Parameters for geometric Brownian motion
    let drift = risk_free_rate - 0.5 * underlying.volatility * underlying.volatility;
    let diffusion = underlying.volatility;
    
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).expect("Failed to create normal distribution");
    
    let mut payoffs = Vec::with_capacity(num_paths);
    
    // Generate Monte Carlo paths
    for _ in 0..num_paths {
        // Generate random shock (standard normal)
        let z = normal.sample(&mut rng);
        
        // Simulate stock price at expiration using geometric Brownian motion
        // S_T = S_0 * exp((r - 0.5*σ²)*T + σ*√T*Z)
        let stock_price_at_expiration = underlying.spot_price
            * (drift * dt + diffusion * sqrt_dt * z).exp();
        
        // Calculate payoff based on option type
        // Call: max(S_T - K, 0), Put: max(K - S_T, 0)
        let payoff = if is_call {
            (stock_price_at_expiration - strike_price).max(0.0)
        } else {
            (strike_price - stock_price_at_expiration).max(0.0)
        };
        payoffs.push(payoff);
    }
    
    // Calculate average payoff
    let average_payoff: f64 = payoffs.iter().sum::<f64>() / num_paths as f64;
    
    // Discount to present value
    let option_price = average_payoff * (-risk_free_rate * time_to_expiration).exp();
    
    option_price
}

fn main() {
    // Example usage
    let underlying = Underlying::new(
        "AAPL".to_string(),
        150.0,  // Current spot price
        0.25,   // 25% annual volatility
    );
    
    let time_horizon_days = 30;
    let strike_price = 155.0;
    let risk_free_rate = 0.05; // 5% annual risk-free rate
    let num_paths = 1000;
    
    // Price a Call option
    let call_price = price_option(
        &underlying,
        time_horizon_days,
        strike_price,
        true,  // is_call = true
        risk_free_rate,
        num_paths,
    );
    
    // Price a Put option
    let put_price = price_option(
        &underlying,
        time_horizon_days,
        strike_price,
        false,  // is_call = false
        risk_free_rate,
        num_paths,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_pricing_call_positive_payoff() {
        // Deep in-the-money call option should have positive value
        let underlying = Underlying::new("TEST".to_string(), 100.0, 0.20);
        let price = price_option(&underlying, 30, 50.0, true, 0.05, 1000);
        assert!(price > 0.0, "Deep ITM call should have positive value");
    }

    #[test]
    fn test_option_pricing_put_positive_payoff() {
        // Deep in-the-money put option should have positive value
        let underlying = Underlying::new("TEST".to_string(), 100.0, 0.20);
        let price = price_option(&underlying, 30, 150.0, false, 0.05, 1000);
        assert!(price > 0.0, "Deep ITM put should have positive value");
    }

    #[test]
    fn test_option_pricing_at_the_money() {
        // At-the-money option should have some value due to time value
        let underlying = Underlying::new("TEST".to_string(), 100.0, 0.20);
        let call_price = price_option(&underlying, 30, 100.0, true, 0.05, 1000);
        let put_price = price_option(&underlying, 30, 100.0, false, 0.05, 1000);
        assert!(call_price >= 0.0, "ATM call should have non-negative value");
        assert!(put_price >= 0.0, "ATM put should have non-negative value");
    }
}

