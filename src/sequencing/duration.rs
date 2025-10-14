/// Musical note duration represented as a rational fraction of a whole note.
/// All operations preserve exact ratios—no floating point drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    /// Numerator: how many parts
    pub numerator: u32,
    /// Denominator: of what size (4 = quarter, 8 = eighth, etc.)
    pub denominator: u32,
}

impl Duration {
    // Standard note values
    pub const WHOLE: Duration = Duration {
        numerator: 1,
        denominator: 1,
    };
    pub const HALF: Duration = Duration {
        numerator: 1,
        denominator: 2,
    };
    pub const QUARTER: Duration = Duration {
        numerator: 1,
        denominator: 4,
    };
    pub const EIGHTH: Duration = Duration {
        numerator: 1,
        denominator: 8,
    };
    pub const SIXTEENTH: Duration = Duration {
        numerator: 1,
        denominator: 16,
    };
    pub const THIRTY_SECOND: Duration = Duration {
        numerator: 1,
        denominator: 32,
    };

    // Convenience constants for common dotted notes
    pub const DOTTED_HALF: Duration = Duration::HALF.dotted();
    pub const DOTTED_QUARTER: Duration = Duration::QUARTER.dotted();
    pub const DOTTED_EIGHTH: Duration = Duration::EIGHTH.dotted();

    // Convenience constants for common triplets
    pub const QUARTER_TRIPLET: Duration = Duration::QUARTER.triplet();
    pub const EIGHTH_TRIPLET: Duration = Duration::EIGHTH.triplet();

    /// Apply a dot: multiply duration by 3/2 (increases by 50%)
    pub const fn dotted(self) -> Self {
        Duration {
            numerator: self.numerator * 3,
            denominator: self.denominator * 2,
        }
    }

    /// Create a triplet: multiply duration by 2/3
    /// (three notes in the time of two)
    pub const fn triplet(self) -> Self {
        self.tuplet(2, 3)
    }

    /// General tuplet: `played` notes in the time of `in_time_of` notes
    /// E.g., `.tuplet(2, 3)` = triplet (3 in time of 2)
    ///       `.tuplet(4, 5)` = quintuplet (5 in time of 4)
    pub const fn tuplet(self, in_time_of: u32, played: u32) -> Self {
        Duration {
            numerator: self.numerator * in_time_of,
            denominator: self.denominator * played,
        }
    }

    /// Double the duration
    pub const fn double(self) -> Self {
        Duration {
            numerator: self.numerator * 2,
            denominator: self.denominator,
        }
    }

    /// Halve the duration
    pub const fn half(self) -> Self {
        Duration {
            numerator: self.numerator,
            denominator: self.denominator * 2,
        }
    }

    /// Reduce the fraction to lowest terms using GCD
    pub const fn reduce(self) -> Self {
        let gcd = const_gcd(self.numerator, self.denominator);
        Duration {
            numerator: self.numerator / gcd,
            denominator: self.denominator / gcd,
        }
    }

    /// Convert this duration to integer ticks
    /// ppq = pulses per quarter note (standard MIDI timing resolution)
    /// Formula: ticks = (numerator * 4 * ppq) / denominator
    pub fn to_ticks(&self, ppq: u32) -> u32 {
        (self.numerator * 4 * ppq) / self.denominator
    }

    /// Add two durations (finds common denominator)
    pub const fn add(self, other: Self) -> Self {
        Duration {
            numerator: self.numerator * other.denominator + other.numerator * self.denominator,
            denominator: self.denominator * other.denominator,
        }
        .reduce()
    }
}

/// Compute greatest common divisor (Euclidean algorithm)
/// Used to reduce fractions to lowest terms
const fn const_gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_durations_to_ticks() {
        let ppq = 480; // Standard MIDI ppq

        // Whole note = 4 quarters = 4 * 480 = 1920 ticks
        assert_eq!(Duration::WHOLE.to_ticks(ppq), 1920);

        // Half note = 2 quarters = 2 * 480 = 960 ticks
        assert_eq!(Duration::HALF.to_ticks(ppq), 960);

        // Quarter note = 1 quarter = 480 ticks
        assert_eq!(Duration::QUARTER.to_ticks(ppq), 480);

        // Eighth note = 0.5 quarter = 240 ticks
        assert_eq!(Duration::EIGHTH.to_ticks(ppq), 240);

        // Sixteenth note = 0.25 quarter = 120 ticks
        assert_eq!(Duration::SIXTEENTH.to_ticks(ppq), 120);
    }

    #[test]
    fn test_dotted_notes() {
        let ppq = 480;

        // Dotted quarter = 1.5 quarters = 720 ticks
        assert_eq!(Duration::QUARTER.dotted().to_ticks(ppq), 720);
        assert_eq!(Duration::DOTTED_QUARTER.to_ticks(ppq), 720);

        // Dotted eighth = 0.75 quarter = 360 ticks
        assert_eq!(Duration::EIGHTH.dotted().to_ticks(ppq), 360);
    }

    #[test]
    fn test_triplets() {
        let ppq = 480;

        // Quarter triplet = 2/3 of quarter = 320 ticks
        assert_eq!(Duration::QUARTER.triplet().to_ticks(ppq), 320);

        // Eighth triplet = 2/3 of eighth = 160 ticks
        assert_eq!(Duration::EIGHTH.triplet().to_ticks(ppq), 160);
    }

    #[test]
    fn test_general_tuplets() {
        let ppq = 480;

        // Quintuplet: 5 eighths in time of 4 eighths
        let quintuplet = Duration::EIGHTH.tuplet(4, 5);
        // Each note = (1/8) * (4/5) of whole = 1/10 of whole
        // = (4 * 480) / 10 = 192 ticks
        assert_eq!(quintuplet.to_ticks(ppq), 192);
    }

    #[test]
    fn test_reduce() {
        // 4/8 should reduce to 1/2
        let d = Duration {
            numerator: 4,
            denominator: 8,
        }
        .reduce();
        assert_eq!(d.numerator, 1);
        assert_eq!(d.denominator, 2);

        // 6/9 should reduce to 2/3
        let d = Duration {
            numerator: 6,
            denominator: 9,
        }
        .reduce();
        assert_eq!(d.numerator, 2);
        assert_eq!(d.denominator, 3);
    }

    #[test]
    fn test_chained_operations() {
        let ppq = 480;

        // Dotted eighth triplet (weird but valid)
        let weird = Duration::EIGHTH.dotted().triplet().reduce();

        // Eighth = 1/8, dotted = 3/16, triplet = (3/16)*(2/3) = 6/48 = 1/8
        // So we end up back at an eighth note!
        assert_eq!(weird.to_ticks(ppq), 240);
    }

    #[test]
    fn test_duration_addition() {
        let ppq = 480;

        // Quarter + eighth = 3/8 of whole
        let sum = Duration::QUARTER.add(Duration::EIGHTH);
        // (1/4) + (1/8) = (2/8) + (1/8) = 3/8
        // ticks = (3 * 4 * 480) / 8 = 720
        assert_eq!(sum.to_ticks(ppq), 720);
    }

    #[test]
    fn test_const_evaluation() {
        // Verify that all operations can be evaluated at compile time
        const WEIRD_DURATION: Duration = Duration::QUARTER.dotted().triplet().reduce();
        assert_eq!(WEIRD_DURATION.numerator, 1);
    }
}
