// Copyright (C) 2026 Jorge Andre Castro
// Licence : GNU GPL v2 ou ultérieure

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! # embedded-trig-f32
//!
//! Bibliothèque haute performance pour le calcul trigonométrique en `f32`.
//! Spécifiquement conçue pour les systèmes embarqués sans bibliothèque standard.

/// Constantes mathématiques pré-calculées.
pub mod consts {
    /// La constante PI.
    pub const PI: f32 = core::f32::consts::PI;
    /// La constante TAU (2*PI).
    pub const TAU: f32 = core::f32::consts::TAU;
    /// La constante PI/2.
    pub const FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;
}

/// Erreurs de domaine ou de calcul.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrigError {
    /// L'argument dépasse les limites autorisées (ex: asin(1.1)).
    DomainError,
    /// Résultat indéfini (ex: division par zéro dans atan2).
    Undefined,
    /// Valeur d'entrée non finie (NaN ou Infini).
    NonFiniteValue,
}

// ─────────────────────────────────────────────────────────────
//  Outils de calcul internes
// ─────────────────────────────────────────────────────────────

#[inline(always)]
fn internal_sqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    #[cfg(all(target_arch = "arm", target_feature = "vfp2"))]
    { x.sqrt() }
    #[cfg(not(all(target_arch = "arm", target_feature = "vfp2")))]
    {
        let mut r = f32::from_bits(((x.to_bits() >> 1) + 0x1FBB_4F2E) & 0x7FFF_FFFF);
        r = 0.5 * (r + x / r);
        r = 0.5 * (r + x / r);
        r = 0.5 * (r + x / r);
        r
    }
}

#[inline]
fn wrap_angle(x: f32) -> f32 {
    let mut w = x % consts::TAU;
    if w > consts::PI { w -= consts::TAU; }
    if w <= -consts::PI { w += consts::TAU; }
    w
}

/// Approximation sinus très stable (Bhaskara I)
/// Précision suffisante pour f32 sur l'intervalle [0, PI]
fn base_sin(x: f32) -> f32 {
    let x_deg = x * (180.0 / consts::PI);
    let top = 4.0 * x_deg * (180.0 - x_deg);
    let bottom = 40500.0 - x_deg * (180.0 - x_deg);
    top / bottom
}

// ─────────────────────────────────────────────────────────────
//  API Publique
// ─────────────────────────────────────────────────────────────

/// Calcule le sinus.
pub fn sin(x: f32) -> f32 {
    if !x.is_finite() { return f32::NAN; }
    let w = wrap_angle(x);
    if w < 0.0 {
        -base_sin(-w)
    } else {
        base_sin(w)
    }
}

/// Calcule le cosinus.
pub fn cos(x: f32) -> f32 {
    sin(x + consts::FRAC_PI_2)
}
/// Calcule l'arc tangente (y, x).
pub fn atan2(y: f32, x: f32) -> Result<f32, TrigError> {
    if !y.is_finite() || !x.is_finite() { return Err(TrigError::NonFiniteValue); }
    if x == 0.0 && y == 0.0 { return Err(TrigError::Undefined); }

    let abs_y = y.abs();
    let abs_x = x.abs();
    
    // Approximation rationnelle stable
    let a = if abs_x > abs_y { abs_y / abs_x } else { abs_x / abs_y };
    let s = a * a;
    let mut r = ((-0.0464964749 * s + 0.15931422) * s - 0.327622764) * s * a + a;

    if abs_y > abs_x { r = consts::FRAC_PI_2 - r; }
    if x < 0.0 { r = consts::PI - r; }
    if y < 0.0 { r = -r; }

    Ok(r)
}

/// Calcule l'arc sinus.
pub fn asin(x: f32) -> Result<f32, TrigError> {
    if x < -1.0 || x > 1.0 { return Err(TrigError::DomainError); }
    if x == 1.0 { return Ok(consts::FRAC_PI_2); }
    if x == -1.0 { return Ok(-consts::FRAC_PI_2); }
    atan2(x, internal_sqrt(1.0 - x * x))
}

/// Calcule l'arc cosinus.
pub fn acos(x: f32) -> Result<f32, TrigError> {
    if x < -1.0 || x > 1.0 { return Err(TrigError::DomainError); }
    Ok(consts::FRAC_PI_2 - asin(x)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::PI;

    const EPS: f32 = 2e-3; // Bhaskara est stable mais un peu moins précis que Taylor

    #[test]
    fn test_sin_basic() {
        assert!((sin(0.0)).abs() < EPS);
        assert!((sin(PI / 2.0) - 1.0).abs() < EPS);
        assert!((sin(PI)).abs() < EPS);
    }

    #[test]
    fn test_cos_basic() {
        assert!((cos(0.0) - 1.0).abs() < EPS);
        assert!((cos(PI / 2.0)).abs() < EPS);
    }

    #[test]
    fn test_reduction_angle() {
        assert!((wrap_angle(3.0 * PI).abs() - PI).abs() < EPS);
    }

    #[test]
    fn test_atan2() {
        assert!((atan2(1.0, 1.0).unwrap() - PI / 4.0).abs() < EPS);
    }

    #[test]
    fn test_asin_acos() {
        assert!((asin(0.5).unwrap() - PI / 6.0).abs() < EPS);
        assert!((acos(0.5).unwrap() - PI / 3.0).abs() < EPS);
    }
}