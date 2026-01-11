use std::error::Error;
use std::fmt;

/// Type of barrier for multi-underlying options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierType {
    /// Worst-Of: barrier is hit if the worst performing underlying hits the barrier
    WorstOf,
    /// Best-Of: barrier is hit if the best performing underlying hits the barrier
    BestOf,
    /// Average: barrier is hit if the average of underlyings hits the barrier
    Average,
    /// Median: barrier is hit if the median of underlyings hits the barrier
    Median,
}

/// Represents a barrier for barrier options
#[derive(Debug, Clone)]
pub struct Barrier {
    /// Barrier level (same unit as strike and spot price, or relative if `relative` is true)
    pub barrier_level: f64,
    /// `true` for "in" barrier (option only has value if barrier was hit),
    /// `false` for "out" barrier (option only has value if barrier was NOT hit)
    pub in_out: bool,
    /// `true` for "up" barrier (barrier is hit if price goes above barrier_level),
    /// `false` for "down" barrier (barrier is hit if price goes below barrier_level)
    pub up_down: bool,
    /// Type of barrier for multi-underlying options
    pub barrier_type: BarrierType,
    /// `true` if barrier_level is relative to current spot/avg/median, `false` for absolute level
    pub relative: bool,
    /// Indices into the list of underlyings this barrier applies to
    pub underlying_indices: Vec<usize>,
}

/// Error type for barrier creation
#[derive(Debug, Clone)]
pub struct BarrierError {
    message: String,
}

impl fmt::Display for BarrierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for BarrierError {}

impl Barrier {
    /// Creates a new barrier for a single underlying
    ///
    /// # Arguments
    /// * `barrier_level` - Barrier level (absolute or relative)
    /// * `in_out` - `true` for "in" barrier, `false` for "out" barrier
    /// * `up_down` - `true` for "up" barrier, `false` for "down" barrier
    /// * `relative` - `true` if barrier_level is relative to spot price
    pub fn new(
        barrier_level: f64,
        in_out: bool,
        up_down: bool,
        relative: bool,
    ) -> Self {
        Self {
            barrier_level,
            in_out,
            up_down,
            barrier_type: BarrierType::WorstOf, // Default for single underlying
            relative,
            underlying_indices: vec![0], // Single underlying at index 0
        }
    }

    /// Creates a new barrier for multiple underlyings
    ///
    /// # Arguments
    /// * `barrier_level` - Barrier level (must be relative if multiple underlyings)
    /// * `in_out` - `true` for "in" barrier, `false` for "out" barrier
    /// * `up_down` - `true` for "up" barrier, `false` for "down" barrier
    /// * `barrier_type` - Type of barrier (WorstOf, BestOf, Average, Median)
    /// * `relative` - `true` if barrier_level is relative to spot/avg/median
    /// * `underlying_indices` - Indices into the list of underlyings this barrier applies to
    ///
    /// # Errors
    /// Returns `BarrierError` if multiple underlyings are specified with an absolute barrier level
    pub fn new_multi(
        barrier_level: f64,
        in_out: bool,
        up_down: bool,
        barrier_type: BarrierType,
        relative: bool,
        underlying_indices: Vec<usize>,
    ) -> Result<Self, BarrierError> {
        // Validate: multiple underlyings with absolute level is not allowed
        if underlying_indices.len() > 1 && !relative {
            return Err(BarrierError {
                message: format!(
                    "Cannot create barrier with {} underlyings and absolute level. Use relative=true for multi-underlying barriers.",
                    underlying_indices.len()
                ),
            });
        }

        Ok(Self {
            barrier_level,
            in_out,
            up_down,
            barrier_type,
            relative,
            underlying_indices,
        })
    }
}

