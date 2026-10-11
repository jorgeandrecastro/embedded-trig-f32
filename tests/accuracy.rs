//! Mesure de l'erreur de chaque fonction contre une référence `f64`.
//!
//! Lancer avec :  cargo test --test accuracy -- --nocapture
//!
//! Fichier à placer dans `tests/accuracy.rs` (test d'intégration : aucune
//! modification de `src/lib.rs` nécessaire).

use embedded_trig_f32::*;

/// Position de `f` sur une échelle entière ordonnée (pour compter les ULP).
fn ordered(f: f32) -> i32 {
    let b = f.to_bits() as i32;
    if b < 0 { i32::MIN.wrapping_sub(b) } else { b }
}

fn ulp_dist(a: f32, b: f32) -> u32 {
    (ordered(a) as i64 - ordered(b) as i64).unsigned_abs() as u32
}

/// Balaye `[lo, hi]` en `n` points et affiche l'erreur max absolue et en ULP.
fn sweep(
    name: &str,
    lo: f32,
    hi: f32,
    n: u32,
    f: impl Fn(f32) -> Option<f32>,
    reference: impl Fn(f64) -> f64,
) {
    let (mut max_abs, mut at_abs) = (0.0f64, 0.0f32);
    let (mut max_ulp, mut at_ulp) = (0u32, 0.0f32);
    let mut skipped = 0u32;

    for i in 0..=n {
        let x = lo + (hi - lo) * (i as f32) / (n as f32);
        let Some(got) = f(x) else {
            skipped += 1;
            continue;
        };
        let want = reference(x as f64);
        let abs = (got as f64 - want).abs();
        if abs > max_abs {
            max_abs = abs;
            at_abs = x;
        }
        let ulp = ulp_dist(got, want as f32);
        if ulp > max_ulp {
            max_ulp = ulp;
            at_ulp = x;
        }
    }

    println!(
        "{name:<22} abs max = {max_abs:.3e} (x = {at_abs:>10.4})   ULP max = {max_ulp:>5} (x = {at_ulp:>10.4})   ignorés = {skipped}"
    );
}

#[test]
fn accuracy_report() {
    println!();
    println!("── Erreur vs f64 (référence exacte sur l'entrée f32) ──");

    sweep("sin  [-100, 100]", -100.0, 100.0, 400_000, |x| Some(sin(x)), f64::sin);
    sweep("cos  [-100, 100]", -100.0, 100.0, 400_000, |x| Some(cos(x)), f64::cos);
    sweep("sin  [-8192, 8192]", -8192.0, 8192.0, 400_000, |x| Some(sin(x)), f64::sin);
    sweep("cos  [-8192, 8192]", -8192.0, 8192.0, 400_000, |x| Some(cos(x)), f64::cos);
    sweep("sincos.0 [-10, 10]", -10.0, 10.0, 200_000, |x| Some(sincos(x).0), f64::sin);
    sweep("sincos.1 [-10, 10]", -10.0, 10.0, 200_000, |x| Some(sincos(x).1), f64::cos);

    // tan / cot : on reste à distance des pôles pour que l'erreur absolue ait un sens.
    sweep("tan  [-1.4, 1.4]", -1.4, 1.4, 200_000, |x| tan(x).ok(), f64::tan);
    sweep("cot  [0.2, 2.9]", 0.2, 2.9, 200_000, |x| cot(x).ok(), |x| 1.0 / x.tan());

    sweep("atan [-50, 50]", -50.0, 50.0, 400_000, |x| Some(atan(x)), f64::atan);
    sweep("atan [-1, 1]", -1.0, 1.0, 200_000, |x| Some(atan(x)), f64::atan);

    sweep("asin [-1, 1]", -1.0, 1.0, 400_000, |x| asin(x).ok(), f64::asin);
    sweep("acos [-1, 1]", -1.0, 1.0, 400_000, |x| acos(x).ok(), f64::acos);
    sweep("asin [0.99, 1]", 0.99, 1.0, 100_000, |x| asin(x).ok(), f64::asin);
    sweep("acos [0.99, 1]", 0.99, 1.0, 100_000, |x| acos(x).ok(), f64::acos);

    // atan2 : on balaye l'angle θ ∈ [-π, π] et on reconstruit (y, x) = (sin θ, cos θ).
    sweep(
        "atan2 (angle -π..π)",
        -3.14,
        3.14,
        200_000,
        |t| {
            let (y, x) = ((t as f64).sin() as f32, (t as f64).cos() as f32);
            atan2(y, x).ok()
        },
        |t| {
            // Référence sur les mêmes (y, x) arrondis en f32.
            let (y, x) = ((t.sin() as f32) as f64, (t.cos() as f32) as f64);
            y.atan2(x)
        },
    );

    println!();
}