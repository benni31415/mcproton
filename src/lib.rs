pub mod barrier;
pub mod underlying;

use nalgebra::{DMatrix, DVector};
use rand_distr::{Distribution, Normal};
pub use barrier::{Barrier, BarrierType};
pub use underlying::Underlying;

/// Prices a European option (Call or Put) using Monte Carlo simulation
/// Supports multiple underlyings with correlation and barrier options.
///
/// # Arguments
/// * `underlyings` - List of underlying assets
/// * `correlation_matrix` - Correlation matrix (n x n) where n is the number of underlyings.
///   Must be symmetric, positive semi-definite, with 1.0 on the diagonal.
/// * `time_horizon_days` - Time to expiration in days
/// * `strike_price` - Strike price of the option
/// * `is_call` - `true` for Call option, `false` for Put option
/// * `risk_free_rate` - Annual risk-free interest rate (as a decimal, e.g., 0.05 for 5%)
/// * `num_paths` - Number of Monte Carlo simulation paths
/// * `barrier` - Optional barrier for barrier options. If `None`, prices a vanilla option.
///
/// # Returns
/// The estimated option price
pub fn price_option(
    underlyings: &[Underlying],
    correlation_matrix: &DMatrix<f64>,
    time_horizon_days: u32,
    strike_price: f64,
    is_call: bool,
    risk_free_rate: f64,
    num_paths: usize,
    barrier: Option<&Barrier>,
) -> f64 {
    let num_underlyings = underlyings.len();
    
    // Validate correlation matrix dimensions
    assert_eq!(
        correlation_matrix.nrows(),
        num_underlyings,
        "Correlation matrix must have {} rows",
        num_underlyings
    );
    assert_eq!(
        correlation_matrix.ncols(),
        num_underlyings,
        "Correlation matrix must have {} columns",
        num_underlyings
    );
    
    let time_to_expiration = time_horizon_days as f64 / 365.0; // Convert days to years
    
    // Compute Cholesky decomposition of correlation matrix for correlated random variables
    let cholesky = correlation_matrix
        .clone()
        .cholesky()
        .expect("Correlation matrix must be positive semi-definite");
    
    // Pre-compute drift and diffusion parameters for each underlying
    let drifts: Vec<f64> = underlyings
        .iter()
        .map(|u| risk_free_rate - 0.5 * u.volatility * u.volatility)
        .collect();
    let diffusions: Vec<f64> = underlyings.iter().map(|u| u.volatility).collect();
    
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).expect("Failed to create normal distribution");
    
    let mut payoffs = Vec::with_capacity(num_paths);
    
    // Determine number of time steps for simulation
    // For barrier options, we need multiple steps to check barrier hits
    // For vanilla options, we can use a single step
    let num_steps = if barrier.is_some() {
        time_horizon_days as usize // Daily steps for barrier checking
    } else {
        1 // Single step for vanilla options
    };
    
    let dt = time_to_expiration / num_steps as f64;
    let sqrt_dt = dt.sqrt();
    
    // Helper function to calculate reference value based on barrier type
    let calculate_reference = |prices: &[f64], indices: &[usize], barrier_type: BarrierType| -> f64 {
        match barrier_type {
            BarrierType::WorstOf => {
                indices
                    .iter()
                    .map(|&idx| prices[idx])
                    .fold(f64::INFINITY, f64::min)
            }
            BarrierType::BestOf => {
                indices
                    .iter()
                    .map(|&idx| prices[idx])
                    .fold(f64::NEG_INFINITY, f64::max)
            }
            BarrierType::Average => {
                let sum: f64 = indices.iter().map(|&idx| prices[idx]).sum();
                sum / indices.len() as f64
            }
            BarrierType::Median => {
                let mut values: Vec<f64> = indices.iter().map(|&idx| prices[idx]).collect();
                values.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mid = values.len() / 2;
                if values.len() % 2 == 0 {
                    (values[mid - 1] + values[mid]) / 2.0
                } else {
                    values[mid]
                }
            }
        }
    };
    
    // Pre-calculate initial reference for relative barriers (once before the loop)
    let initial_reference = if let Some(barrier) = barrier {
        if barrier.relative {
            let initial_prices: Vec<f64> = underlyings.iter().map(|u| u.spot_price).collect();
            Some(calculate_reference(&initial_prices, &barrier.underlying_indices, barrier.barrier_type))
        } else {
            None
        }
    } else {
        None
    };
    
    // Generate Monte Carlo paths
    for _ in 0..num_paths {
        let mut current_prices: Vec<f64> = underlyings.iter().map(|u| u.spot_price).collect();
        let mut barrier_hit = false;
        
        // Simulate path step by step
        for _ in 0..num_steps {
            // Generate independent standard normal random variables
            let z_independent = DVector::from_iterator(
                num_underlyings,
                (0..num_underlyings).map(|_| normal.sample(&mut rng)),
            );
            
            // Transform to correlated random variables using Cholesky decomposition
            let z_correlated = cholesky.l() * z_independent;
            
            // Update prices for each underlying using geometric Brownian motion
            for i in 0..num_underlyings {
                // S_{t+dt} = S_t * exp((r - 0.5*σ²)*dt + σ*√dt*Z)
                current_prices[i] = current_prices[i]
                    * (drifts[i] * dt + diffusions[i] * sqrt_dt * z_correlated[i]).exp();
            }
            
            // Check if barrier was hit (only if barrier exists)
            if let Some(barrier) = barrier {
                // Calculate the effective barrier level
                let effective_barrier_level = if barrier.relative {
                    // For relative barriers, multiply initial reference by barrier_level
                    initial_reference.unwrap() * barrier.barrier_level
                } else {
                    barrier.barrier_level
                };
                
                // Calculate the current comparison value based on barrier type
                let comparison_value = calculate_reference(
                    &current_prices,
                    &barrier.underlying_indices,
                    barrier.barrier_type,
                );
                
                let hit = if barrier.up_down {
                    // Up barrier: hit if value goes above barrier level
                    comparison_value >= effective_barrier_level
                } else {
                    // Down barrier: hit if value goes below barrier level
                    comparison_value <= effective_barrier_level
                };
                
                if hit {
                    barrier_hit = true;
                }
            }
        }
        
        // Calculate payoff based on option type
        // For multi-underlying, use the first underlying's price (can be extended)
        // Call: max(S_T - K, 0), Put: max(K - S_T, 0)
        let final_price = current_prices[0]; // Using first underlying for payoff
        let intrinsic_payoff = if is_call {
            (final_price - strike_price).max(0.0)
        } else {
            (strike_price - final_price).max(0.0)
        };
        
        // Apply barrier logic if barrier exists
        let payoff = if let Some(barrier) = barrier {
            if barrier.in_out {
                // "In" barrier: option only has value if barrier was hit
                if barrier_hit {
                    intrinsic_payoff
                } else {
                    0.0
                }
            } else {
                // "Out" barrier: option only has value if barrier was NOT hit
                if barrier_hit {
                    0.0
                } else {
                    intrinsic_payoff
                }
            }
        } else {
            // Vanilla option: no barrier logic
            intrinsic_payoff
        };
        
        payoffs.push(payoff);
    }
    
    // Calculate average payoff
    let average_payoff: f64 = payoffs.iter().sum::<f64>() / num_paths as f64;
    
    // Discount to present value
    let option_price = average_payoff * (-risk_free_rate * time_to_expiration).exp();
    
    option_price
}

