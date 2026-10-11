# embedded-trig-f32

[![Crate](https://img.shields.io/badge/rust-no__std-orange.svg)]()
[![License](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)]()
[![Target](https://img.shields.io/badge/target-Cortex--M0%2B%20%7C%20M4F%20%7C%20M33%20%7C%20M7-green.svg)]()

**embedded-trig-f32** est une bibliothèque de fonctions trigonométriques `f32` écrite en **Rust pur**, pour les systèmes embarqués `no_std`. Elle n'utilise ni `libm` ni `micromath` : seule dépendance, la crate sœur [`embedded-f32-sqrt`](https://crates.io/crates/embedded-f32-sqrt) pour la racine carrée.

La version 0.3 remplace l'approximation de Bhaskara I par une **réduction d'argument par quadrant** et des **polynômes minimax**, ce qui ramène l'erreur de ~1,6·10⁻³ à l'ordre de 10⁻⁷. Voir le [CHANGELOG](CHANGELOG.md).

## 🚀 Points forts

* **Précis** : réduction de Cody-Waite (π/2 en trois constantes) et polynômes minimax de type Cephes pour `sin`, `cos` et `atan`.
* **Efficace** : `sincos` ne fait qu'**une seule** réduction d'argument ; `sin` et `cos` ont chacun un chemin direct.
* **Une seule dépendance** : `embedded-f32-sqrt` (Newton-Raphson, déterministe, sans FPU requise).
* **Sûr** : `#![forbid(unsafe_code)]`, `Result<f32, TrigError>` pour les fonctions qui ont des points indéfinis, gestion explicite de NaN et ±∞.
* **Portable** : fonctionne sur Cortex-M0+ (sans FPU) comme sur M4F / M33 / M7.

## 🛠 Installation

```toml
[dependencies]
embedded-trig-f32 = "0.3"
```

Aucune option de compilation particulière n'est nécessaire. Pour une cible avec FPU (RP2350, STM32…), utilisez simplement la cible `hard-float` correspondante, par exemple `thumbv8m.main-none-eabihf` ou `thumbv7em-none-eabihf`.

---

## 📖 Fonctions disponibles

| Fonction      | Description                          | Retour                   |
|---------------|--------------------------------------|--------------------------|
| `sin(x)`      | Sinus (radians)                      | `f32`                    |
| `cos(x)`      | Cosinus (radians)                    | `f32`                    |
| `sincos(x)`   | Sinus et cosinus (une réduction)     | `(f32, f32)`             |
| `tan(x)`      | Tangente                             | `Result<f32, TrigError>` |
| `cot(x)`      | Cotangente                           | `Result<f32, TrigError>` |
| `atan(x)`     | Arc tangente, `[-π/2, π/2]`          | `f32`                    |
| `atan2(y, x)` | Arc tangente 2, `[-π, π]`            | `Result<f32, TrigError>` |
| `asin(x)`     | Arc sinus, `[-π/2, π/2]`             | `Result<f32, TrigError>` |
| `acos(x)`     | Arc cosinus, `[0, π]`                | `Result<f32, TrigError>` |

### Comportement aux limites

* `sin`, `cos`, `sincos` renvoient `NaN` si l'entrée n'est pas finie.
* `tan` / `cot` renvoient `Err(Undefined)` près d'un pôle (`|cos|` ou `|sin|` < 1e-7), par exemple `tan(PI/2)`, `cot(0.0)`.
* `atan2(0.0, 0.0)` renvoie `Err(Undefined)` (la `std` renvoie 0). `atan2` gère `±0.0` : `atan2(-0.0, -1.0) = -π`.
* `asin` / `acos` renvoient `Err(DomainError)` si `|x| > 1`, et `Err(NonFiniteValue)` pour NaN / ±∞.
* `atan` est infaillible : `atan(±∞) = ±π/2`, `atan(NaN) = NaN`.

---

## 💻 Exemple d'utilisation

```rust
use embedded_trig_f32::{atan2, sincos, tan, consts::PI};

fn example() {
    let angle = PI / 4.0; // 45°

    let (s, c) = sincos(angle);
    let t = tan(angle).unwrap_or(0.0);

    // Calcul inverse sécurisé
    match atan2(s, c) {
        Ok(res) => {
            // res est proche de PI/4
            let _ = (res, t);
        }
        Err(_e) => {
            // NaN, infini ou point indéfini
        }
    }
}
```

---

## 🧪 Précision & Tests

Erreur mesurée contre une référence `f64` (balayages de 100 000 à 400 000 points) :

| Fonction            | Plage testée    | Erreur abs. max | Erreur max (ULP) |
|---------------------|-----------------|-----------------|------------------|
| `sin`               | [-100, 100]     | 8,8·10⁻⁸        | 1                |
| `cos`               | [-100, 100]     | 8,9·10⁻⁸        | 1                |
| `sin` / `cos`       | [-8192, 8192]   | 9,0·10⁻⁸        | 45 (près des zéros) |
| `sincos`            | [-10, 10]       | 8,9·10⁻⁸        | 1                |
| `tan`               | [-1,4, 1,4]     | 7,3·10⁻⁷        | 3                |
| `cot`               | [0,2, 2,9]      | 6,5·10⁻⁷        | 3                |
| `atan`              | [-50, 50]       | 1,3·10⁻⁷        | 2                |
| `atan2`             | tour complet    | 2,7·10⁻⁷        | 3                |
| `asin`              | [-1, 1]         | 1,6·10⁻⁷        | 4                |
| `acos`              | [-1, 1]         | 2,8·10⁻⁷        | 3                |

* L'erreur absolue de `sin`/`cos` est de l'ordre de la précision du `f32` lui-même (~6·10⁻⁸ pour une valeur proche de 1). Le nombre d'ULP plus élevé sur [-8192, 8192] ne se produit que près des zéros de la fonction, où l'erreur absolue reste de ~9·10⁻⁸.
* **Grands arguments** : précision maximale pour `|x| ≤ 8192`. Au-delà, l'argument est d'abord réduit par `x % TAU` : le résultat reste borné dans `[-1, 1]` mais perd en précision (limite inhérente au `f32`).
* `tan` et `cot` grossissent l'erreur à l'approche de leurs pôles (comportement mathématique normal).

### Reproduire les mesures

```bash
cargo test                                  # tests unitaires et balayages
cargo test --test accuracy -- --nocapture   # rapport d'erreur par fonction
```

Les tests s'exécutent sur la machine hôte avec `std` ; la bibliothèque, elle, reste `no_std`.

---

## ⚖️ Licence

Ce programme est un logiciel libre ; vous pouvez le redistribuer et/ou le modifier selon les termes de la Licence Publique Générale GNU (GPL) version 2 ou ultérieure.

**Copyright (C) 2026 Jorge Andre Castro**