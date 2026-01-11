/// Represents an underlying asset for option pricing
#[derive(Debug, Clone)]
pub struct Underlying {
    /// Name of the underlying asset
    pub name: String,
    /// Current spot price
    pub spot_price: f64,
    /// Volatility (annualized, as a decimal, e.g., 0.20 for 20%)
    pub volatility: f64,
}

impl Underlying {
    /// Creates a new underlying asset
    pub fn new(name: String, spot_price: f64, volatility: f64) -> Self {
        Self {
            name,
            spot_price,
            volatility,
        }
    }
}


