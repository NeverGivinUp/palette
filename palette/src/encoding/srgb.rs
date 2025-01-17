//! The sRGB standard.

use crate::{
    bool_mask::LazySelect,
    encoding::{FromLinear, IntoLinear},
    luma::LumaStandard,
    num::{Arithmetics, MulAdd, MulSub, PartialCmp, Powf, Real},
    rgb::{Primaries, RgbSpace, RgbStandard},
    white_point::{Any, D65},
    Yxy,
};

use lookup_tables::*;

mod lookup_tables;

/// The sRGB standard, color space, and transfer function.
///
/// # As transfer function
///
/// `Srgb` will not use any kind of approximation when converting from `T` to
/// `T`. This involves calls to `powf`, which may make it too slow for certain
/// applications.
///
/// There are some specialized cases where it has been optimized:
///
/// * When converting from `u8` to `f32` or `f64`, while converting to linear
///   space. This uses lookup tables with precomputed values. `f32` will use the
///   table provided by [fast_srgb8::srgb8_to_f32].
/// * When converting from `f32` or `f64` to `u8`, while converting from linear
///   space. This uses [fast_srgb8::f32_to_srgb8].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Srgb;

impl<T: Real> Primaries<T> for Srgb {
    fn red() -> Yxy<Any, T> {
        Yxy::new(
            T::from_f64(0.6400),
            T::from_f64(0.3300),
            T::from_f64(0.212656),
        )
    }
    fn green() -> Yxy<Any, T> {
        Yxy::new(
            T::from_f64(0.3000),
            T::from_f64(0.6000),
            T::from_f64(0.715158),
        )
    }
    fn blue() -> Yxy<Any, T> {
        Yxy::new(
            T::from_f64(0.1500),
            T::from_f64(0.0600),
            T::from_f64(0.072186),
        )
    }
}

impl RgbSpace for Srgb {
    type Primaries = Srgb;
    type WhitePoint = D65;
}

impl RgbStandard for Srgb {
    type Space = Srgb;
    type TransferFn = Srgb;
}

impl LumaStandard for Srgb {
    type WhitePoint = D65;
    type TransferFn = Srgb;
}

impl<T> IntoLinear<T, T> for Srgb
where
    T: Real + Powf + MulAdd + Arithmetics + PartialCmp + Clone,
    T::Mask: LazySelect<T>,
{
    #[inline]
    fn into_linear(x: T) -> T {
        // Dividing the constants directly shows performance benefits in benchmarks for this function
        lazy_select! {
            if x.lt_eq(&T::from_f64(0.04045)) => T::from_f64(1.0 / 12.92) * &x,
            else => x.clone().mul_add(T::from_f64(1.0 / 1.055), T::from_f64(0.055 / 1.055)).powf(T::from_f64(2.4)),
        }
    }
}

impl<T> FromLinear<T, T> for Srgb
where
    T: Real + Powf + MulSub + Arithmetics + PartialCmp + Clone,
    T::Mask: LazySelect<T>,
{
    #[inline]
    fn from_linear(x: T) -> T {
        lazy_select! {
            if x.lt_eq(&T::from_f64(0.0031308)) => T::from_f64(12.92) * &x,
            else => x.clone().powf(T::from_f64(1.0 / 2.4)).mul_sub(T::from_f64(1.055), T::from_f64(0.055)),
        }
    }
}

impl IntoLinear<f32, u8> for Srgb {
    #[inline]
    fn into_linear(encoded: u8) -> f32 {
        fast_srgb8::srgb8_to_f32(encoded)
    }
}

impl FromLinear<f32, u8> for Srgb {
    #[inline]
    fn from_linear(linear: f32) -> u8 {
        fast_srgb8::f32_to_srgb8(linear)
    }
}

impl IntoLinear<f64, u8> for Srgb {
    #[inline]
    fn into_linear(encoded: u8) -> f64 {
        SRGB_U8_TO_F64[encoded as usize]
    }
}

impl FromLinear<f64, u8> for Srgb {
    #[inline]
    fn from_linear(linear: f64) -> u8 {
        Srgb::from_linear(linear as f32)
    }
}

#[cfg(test)]
mod test {
    use crate::encoding::{FromLinear, IntoLinear, Srgb};

    #[test]
    fn u8_to_f32_to_u8() {
        for expected in 0u8..=255u8 {
            let linear: f32 = Srgb::into_linear(expected);
            let result: u8 = Srgb::from_linear(linear);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn u8_to_f64_to_u8() {
        for expected in 0u8..=255u8 {
            let linear: f64 = Srgb::into_linear(expected);
            let result: u8 = Srgb::from_linear(linear);
            assert_eq!(result, expected);
        }
    }
}
