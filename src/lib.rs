// Copyright (C) 2026 Jorge Andre Castro
// Licence : GNU GPL v2 ou ultérieure

// `cfg_attr(not(test), no_std)` : les tests tournent sur l'hôte avec `std`
// (nécessaire pour comparer à `f64::sin`, etc.), la bibliothèque reste `no_std`.
#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! # embedded-trig-f32
//!
//! Trigonométrie `f32` pour systèmes embarqués `no_std`.
//!
//! - Réduction d'argument par quadrant (Cody-Waite), une seule fois par appel.
//! - Polynômes minimax (type Cephes) : erreur absolue ~1e-7 pour `sin`/`cos`/`atan`.
//! - Racine carrée fournie par [`embedded-f32-sqrt`](https://crates.io/crates/embedded-f32-sqrt).
//!
//! ## Précision et domaine
//!
//! `sin`, `cos`, `sincos` : précision maximale pour `|x| <= 8192`. Au-delà,
//! l'argument est d'abord réduit par `x % TAU` : le résultat reste borné dans
//! `[-1, 1]` mais perd en précision (inévitable en `f32`).

use embedded_f32_sqrt::sqrt as fsqrt;

/// Constantes mathématiques pré-calculées.
pub mod consts {
    /// La constante PI.
    pub const PI: f32 = core::f32::consts::PI;
    /// La constante TAU (2*PI).
    pub const TAU: f32 = core::f32::consts::TAU;
    /// La constante PI/2.
    pub const FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;
    /// La constante PI/4.
    pub const FRAC_PI_4: f32 = core::f32::consts::FRAC_PI_4;
    /// La constante 2/PI.
    pub const FRAC_2_PI: f32 = core::f32::consts::FRAC_2_PI;
}

/// Erreurs de domaine ou de calcul.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TrigError {
    /// L'argument dépasse les limites autorisées (ex: asin(1.1)).
    DomainError,
    /// Résultat indéfini (ex: tan(PI/2), atan2(0, 0)).
    Undefined,
    /// Valeur d'entrée non finie (NaN ou Infini).
    NonFiniteValue,
}

// ─────────────────────────────────────────────────────────────
//  Outils internes
// ─────────────────────────────────────────────────────────────

/// Au-delà, `q * P1` n'est plus exact et la réduction perd sa précision.
const MAX_ACCURATE_ARG: f32 = 8192.0;

/// Seuil sous lequel cos (ou sin) est considéré nul pour `tan` (ou `cot`).
/// `cos(PI/2)` en f32 vaut ~ -4.4e-8, jamais exactement 0.
const POLE_EPS: f32 = 1.0e-7;

// PI/2 découpé en trois constantes (Cody-Waite).
const P1: f32 = 1.570_312_5;
const P2: f32 = 4.837_512_97e-4;
const P3: f32 = 7.549_789_95e-8;

const TAN_PI_8: f32 = 0.414_213_57;

/// Réduit `x` en `(r, q)` avec `x ≈ q·π/2 + r`, `r ∈ [-π/4, π/4]`, `q mod 4`.
#[inline]
fn reduce(x: f32) -> (f32, u8) {
    let x = if x.abs() > MAX_ACCURATE_ARG { x % consts::TAU } else { x };
    let half = if x < 0.0 { -0.5 } else { 0.5 };
    let q = (x * consts::FRAC_2_PI + half) as i32; // arrondi au plus proche
    let qf = q as f32;
    let r = ((x - qf * P1) - qf * P2) - qf * P3;
    (r, (q & 3) as u8)
}

#[inline]
fn sin_poly(r: f32) -> f32 {
    let z = r * r;
    r + r * z * ((-1.951_529_6e-4 * z + 8.332_161e-3) * z - 1.666_665_5e-1)
}

#[inline]
fn cos_poly(r: f32) -> f32 {
    let z = r * r;
    1.0 - 0.5 * z + z * z * ((2.443_315_7e-5 * z - 1.388_731_6e-3) * z + 4.166_664_6e-2)
}

/// atan sur `[0, tan(π/8)]` (polynôme minimax).
#[inline]
fn atan_poly(x: f32) -> f32 {
    let z = x * x;
    (((8.053_744_5e-2 * z - 1.387_768_6e-1) * z + 1.997_771_1e-1) * z - 3.333_295e-1) * z * x + x
}

/// atan sur `[0, 1]`.
#[inline]
fn atan_unit(a: f32) -> f32 {
    if a > TAN_PI_8 {
        consts::FRAC_PI_4 + atan_poly((a - 1.0) / (a + 1.0))
    } else {
        atan_poly(a)
    }
}

/// `sqrt` via `embedded-f32-sqrt`; l'argument est toujours >= 0 ici.
#[inline]
fn sqrt_pos(x: f32) -> f32 {
    fsqrt(x).unwrap_or(0.0)
}

// ─────────────────────────────────────────────────────────────
//  API publique
// ─────────────────────────────────────────────────────────────

/// Calcule le sinus (radians). Renvoie `NaN` si `x` n'est pas fini.
#[inline]
pub fn sin(x: f32) -> f32 {
    if !x.is_finite() {
        return f32::NAN;
    }
    let (r, q) = reduce(x);
    match q {
        0 => sin_poly(r),
        1 => cos_poly(r),
        2 => -sin_poly(r),
        _ => -cos_poly(r),
    }
}

/// Calcule le cosinus (radians). Renvoie `NaN` si `x` n'est pas fini.
#[inline]
pub fn cos(x: f32) -> f32 {
    if !x.is_finite() {
        return f32::NAN;
    }
    let (r, q) = reduce(x);
    match q {
        0 => cos_poly(r),
        1 => -sin_poly(r),
        2 => -cos_poly(r),
        _ => sin_poly(r),
    }
}

/// Calcule sinus et cosinus avec **une seule** réduction d'argument.
#[inline]
pub fn sincos(x: f32) -> (f32, f32) {
    if !x.is_finite() {
        return (f32::NAN, f32::NAN);
    }
    let (r, q) = reduce(x);
    let (s, c) = (sin_poly(r), cos_poly(r));
    match q {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    }
}

/// Calcule la tangente.
///
/// # Errors
/// - [`TrigError::NonFiniteValue`] si `x` est NaN ou infini.
/// - [`TrigError::Undefined`] si `x` est (numériquement) un multiple impair de π/2.
#[inline]
pub fn tan(x: f32) -> Result<f32, TrigError> {
    if !x.is_finite() {
        return Err(TrigError::NonFiniteValue);
    }
    let (s, c) = sincos(x);
    if c.abs() < POLE_EPS {
        return Err(TrigError::Undefined);
    }
    Ok(s / c)
}

/// Calcule la cotangente.
///
/// # Errors
/// - [`TrigError::NonFiniteValue`] si `x` est NaN ou infini.
/// - [`TrigError::Undefined`] si `x` est (numériquement) un multiple de π.
#[inline]
pub fn cot(x: f32) -> Result<f32, TrigError> {
    if !x.is_finite() {
        return Err(TrigError::NonFiniteValue);
    }
    let (s, c) = sincos(x);
    if s.abs() < POLE_EPS {
        return Err(TrigError::Undefined);
    }
    Ok(c / s)
}

/// Calcule l'arc tangente, résultat dans `[-π/2, π/2]`.
///
/// Infaillible : `atan(±∞) = ±π/2`, `atan(NaN) = NaN`.
#[inline]
pub fn atan(x: f32) -> f32 {
    if x.is_nan() {
        return f32::NAN;
    }
    let a = x.abs();
    let r = if a > 1.0 {
        consts::FRAC_PI_2 - atan_unit(1.0 / a)
    } else {
        atan_unit(a)
    };
    if x.is_sign_negative() { -r } else { r }
}

/// Calcule l'arc tangente de `y / x` en tenant compte du quadrant, résultat dans `[-π, π]`.
///
/// # Errors
/// - [`TrigError::NonFiniteValue`] si `x` ou `y` n'est pas fini.
/// - [`TrigError::Undefined`] si `x == 0` et `y == 0` (la `std` renvoie 0 ici).
#[inline]
pub fn atan2(y: f32, x: f32) -> Result<f32, TrigError> {
    if !y.is_finite() || !x.is_finite() {
        return Err(TrigError::NonFiniteValue);
    }
    if x == 0.0 && y == 0.0 {
        return Err(TrigError::Undefined);
    }

    let abs_y = y.abs();
    let abs_x = x.abs();
    let a = if abs_x > abs_y { abs_y / abs_x } else { abs_x / abs_y };
    let mut r = atan_unit(a);

    if abs_y > abs_x {
        r = consts::FRAC_PI_2 - r;
    }
    if x.is_sign_negative() {
        r = consts::PI - r;
    }
    if y.is_sign_negative() {
        r = -r;
    }
    Ok(r)
}

/// Calcule l'arc sinus, résultat dans `[-π/2, π/2]`.
///
/// # Errors
/// - [`TrigError::NonFiniteValue`] si `x` est NaN ou infini.
/// - [`TrigError::DomainError`] si `|x| > 1`.
#[inline]
pub fn asin(x: f32) -> Result<f32, TrigError> {
    if !x.is_finite() {
        return Err(TrigError::NonFiniteValue);
    }
    if !(-1.0..=1.0).contains(&x) {
        return Err(TrigError::DomainError);
    }
    if x == 1.0 {
        return Ok(consts::FRAC_PI_2);
    }
    if x == -1.0 {
        return Ok(-consts::FRAC_PI_2);
    }
    // (1-x)(1+x) évite la perte de précision de 1 - x² près de ±1.
    atan2(x, sqrt_pos((1.0 - x) * (1.0 + x)))
}

/// Calcule l'arc cosinus, résultat dans `[0, π]`.
///
/// # Errors
/// - [`TrigError::NonFiniteValue`] si `x` est NaN ou infini.
/// - [`TrigError::DomainError`] si `|x| > 1`.
#[inline]
pub fn acos(x: f32) -> Result<f32, TrigError> {
    if !x.is_finite() {
        return Err(TrigError::NonFiniteValue);
    }
    if !(-1.0..=1.0).contains(&x) {
        return Err(TrigError::DomainError);
    }
    // atan2(sqrt(1-x²), x) : pas d'annulation catastrophique près de x = 1
    // (contrairement à π/2 - asin(x)).
    atan2(sqrt_pos((1.0 - x) * (1.0 + x)), x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::PI;

    /// Tolérance de départ : à resserrer après avoir mesuré l'erreur max réelle.
    const EPS: f32 = 1e-6;

    fn max_err(lo: f32, hi: f32, n: u32, f: impl Fn(f32) -> f32, reference: impl Fn(f64) -> f64) -> f32 {
        let mut worst = 0.0f32;
        for i in 0..=n {
            let x = lo + (hi - lo) * (i as f32) / (n as f32);
            let e = (f(x) as f64 - reference(x as f64)).abs() as f32;
            if e > worst {
                worst = e;
            }
        }
        worst
    }

    #[test]
    fn sweep_sin_cos() {
        let e = max_err(-100.0, 100.0, 200_000, sin, f64::sin);
        assert!(e < EPS, "sin: erreur max = {e}");
        let e = max_err(-100.0, 100.0, 200_000, cos, f64::cos);
        assert!(e < EPS, "cos: erreur max = {e}");
    }

    #[test]
    fn sweep_sincos_matches() {
        for i in -2000..=2000 {
            let x = i as f32 * 0.01;
            let (s, c) = sincos(x);
            assert!((s - sin(x)).abs() < 1e-7 && (c - cos(x)).abs() < 1e-7);
            assert!((s * s + c * c - 1.0).abs() < 2e-6);
        }
    }

    #[test]
    fn sweep_atan() {
        let e = max_err(-50.0, 50.0, 100_000, atan, f64::atan);
        assert!(e < EPS, "atan: erreur max = {e}");
    }

    #[test]
    fn atan2_four_quadrants_and_zeros() {
        for &(y, x) in &[(1.0f32, 1.0f32), (1.0, -1.0), (-1.0, -1.0), (-1.0, 1.0), (0.0, -1.0), (-0.0, -1.0), (1.0, 0.0), (-1.0, 0.0), (3.0, 0.5)] {
            let got = atan2(y, x).unwrap();
            let want = (y as f64).atan2(x as f64) as f32;
            assert!((got - want).abs() < EPS, "atan2({y},{x}) = {got}, attendu {want}");
        }
        assert_eq!(atan2(0.0, 0.0), Err(TrigError::Undefined));
        assert_eq!(atan2(f32::NAN, 1.0), Err(TrigError::NonFiniteValue));
    }

    #[test]
    fn sweep_asin_acos() {
        for i in -1000..=1000 {
            let x = i as f32 / 1000.0;
            let a = asin(x).unwrap();
            let c = acos(x).unwrap();
            assert!((a as f64 - (x as f64).asin()).abs() < 5e-6, "asin({x})");
            assert!((c as f64 - (x as f64).acos()).abs() < 5e-6, "acos({x})");
        }
        assert_eq!(asin(1.0).unwrap(), consts::FRAC_PI_2);
        assert_eq!(acos(1.0).unwrap(), 0.0);
        assert!((acos(-1.0).unwrap() - PI).abs() < EPS);
        assert_eq!(asin(1.1), Err(TrigError::DomainError));
        assert_eq!(acos(-1.1), Err(TrigError::DomainError));
        assert_eq!(asin(f32::NAN), Err(TrigError::NonFiniteValue));
        assert_eq!(acos(f32::NAN), Err(TrigError::NonFiniteValue));
    }

    #[test]
    fn tan_basic_and_poles() {
        assert!(tan(0.0).unwrap().abs() < EPS);
        assert!((tan(PI / 4.0).unwrap() - 1.0).abs() < 1e-5);
        assert_eq!(tan(PI / 2.0), Err(TrigError::Undefined));
        assert_eq!(tan(f32::NAN), Err(TrigError::NonFiniteValue));
        assert_eq!(tan(f32::INFINITY), Err(TrigError::NonFiniteValue));
    }

    #[test]
    fn cot_basic_and_poles() {
        assert!((cot(PI / 4.0).unwrap() - 1.0).abs() < 1e-5);
        assert!(cot(PI / 2.0).unwrap().abs() < EPS);
        assert_eq!(cot(0.0), Err(TrigError::Undefined));
        assert_eq!(cot(PI), Err(TrigError::Undefined));
        assert_eq!(cot(f32::NAN), Err(TrigError::NonFiniteValue));
    }

    #[test]
    fn symmetry_and_non_finite() {
        for i in 1..1000 {
            let x = i as f32 * 0.07;
            assert!((sin(-x) + sin(x)).abs() < 1e-7);
            assert!((cos(-x) - cos(x)).abs() < 1e-7);
        }
        assert!(sin(f32::NAN).is_nan() && cos(f32::INFINITY).is_nan());
        let (s, c) = sincos(f32::NAN);
        assert!(s.is_nan() && c.is_nan());
    }

    #[test]
    fn huge_arguments_stay_bounded() {
        for &x in &[1.0e4f32, 1.0e9, 1.0e30, -1.0e30, f32::MAX] {
            let (s, c) = sincos(x);
            assert!(s.abs() <= 1.0001 && c.abs() <= 1.0001, "x = {x}");
        }
    }
}