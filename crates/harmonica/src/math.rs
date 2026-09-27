//! Floating-point helpers that work with and without `std`.
//!
//! `f64::sqrt`, `f64::exp`, `f64::sin` and `f64::cos` are inherent methods
//! provided by `std`; they do not exist in `core`. With the `std` feature the
//! helpers forward to those methods, and without it they use `libm`. All
//! transcendental math in this crate must go through these helpers so that
//! `default-features = false` builds keep compiling.

#[cfg(feature = "std")]
#[inline]
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

#[cfg(not(feature = "std"))]
#[inline]
pub fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

#[cfg(feature = "std")]
#[inline]
pub fn exp(x: f64) -> f64 {
    x.exp()
}

#[cfg(not(feature = "std"))]
#[inline]
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

#[cfg(feature = "std")]
#[inline]
pub fn sin(x: f64) -> f64 {
    x.sin()
}

#[cfg(not(feature = "std"))]
#[inline]
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

#[cfg(feature = "std")]
#[inline]
pub fn cos(x: f64) -> f64 {
    x.cos()
}

#[cfg(not(feature = "std"))]
#[inline]
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}
