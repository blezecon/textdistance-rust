//! Type aliases matching Python textdistance.algorithms.types

/// Similarity function alias: takes references to two elements and returns similarity as f64.
pub type SimFunc<T> = Option<fn(&T, &T) -> f64>;

/// Element equality test function alias: takes references to two elements and returns bool.
pub type TestFunc<T> = Option<fn(&T, &T) -> bool>;
