#[derive(Clone)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub const EMPTY: Self = Self::new(f64::INFINITY, f64::NEG_INFINITY);
    pub const UNIVERSE: Self = Self::new(f64::NEG_INFINITY, f64::INFINITY);

    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    pub const fn from_intervals(a: &Interval, b: &Interval) -> Self {
        let min = if a.min <= b.min { a.min } else { b.min };
        let max = if a.max >= b.max { a.max } else { b.max };

        Self::new(min, max)
    }

    pub const fn size(&self) -> f64 {
        self.max - self.min
    }

    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    pub fn clamp(&self, x: f64) -> f64 {
        match x {
            x if x < self.min => self.min,
            x if x > self.max => self.max,
            _ => x,
        }
    }

    pub const fn expand(&self, delta: f64) -> Self {
        let padding = delta / 2.0;
        Self::new(self.min - padding, self.max + padding)
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::EMPTY
    }
}
