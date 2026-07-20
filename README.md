# embedded-trig-f32

[![Crate](https://img.shields.io/badge/rust-no__std-orange.svg)]()
[![License](https://img.shields.io/badge/license-GPLv2-blue.svg)]()
[![Target](https://img.shields.io/badge/target-Cortex--M33%20%7C%20M4F%20%7C%20M7-green.svg)]()

**embedded-trig-f32** est une bibliothèque de fonctions trigonométriques `f32` ultra-rapide, écrite en **Pur Rust**, sans aucune dépendance (`zero-deps`) et optimisée pour les systèmes embarqués `no_std`.

La version 0.2 ajoute les fonctions `tan`, `cot` et `sincos` à l’API, tout en conservant les approximations rapides et la sécurité des fonctions inverses.

Consulte le [CHANGELOG](CHANGELOG.md) pour voir les détails de cette version.

## 🚀 Points forts

*   **Zéro Dépendance** : Pas besoin de `libm`.
*   **Hardware-Aware** : Utilise l'instruction FPU `VSQRT` si disponible, sinon bascule sur Newton-Raphson.
*   **Haute Performance** : Utilise des approximations rationnelles (Bhaskara I) et polynomiales (McCutcheon) pour un compromis idéal vitesse/précision.
*   **Sécurité** : Retourne des `Result<f32, TrigError>` pour les fonctions inverses, évitant les crashs silencieux.
*   **#![forbid(unsafe_code)]** : Pour une safety .

## 🛠 Installation

Ajoute ceci à ton fichier `Cargo.toml` :

```toml
[dependencies]
embedded-trig-f32 = "0.2"

```

**Pour profiter de la puissance de la FPU sur ton Cortex-M33**(comme sur le RP2350) ou STM32, configure ton .cargo/config.toml :
```
[target.thumbv8m.main-none-eabihf]
rustflags = ["-C", "target-cpu=cortex-m33", "-C", "target-feature=+vfp2"]

```

----

# 📖 Fonctions disponibles

| Fonction       | Description          | Retour                         |
|----------------|----------------------|--------------------------------|
| `sin(x)`       | Sinus (radians)      | `f32`                          |
| `cos(x)`       | Cosinus (radians)    | `f32`                          |
| `tan(x)`       | Tangente             | `Result<f32, TrigError>`       |
| `cot(x)`       | Cotangente           | `Result<f32, TrigError>`       |
| `sincos(x)`    | Sinus et cosinus     | `(f32, f32)`                   |
| `atan2(y, x)`  | Arc tangente 2       | `Result<f32, TrigError>`       |
| `asin(x)`      | Arc sinus            | `Result<f32, TrigError>`       |
| `acos(x)`      | Arc cosinus          | `Result<f32, TrigError>`       |

----

# 💻 Exemple d'utilisation

**Voici comment l'utiliser dans un projet de contrôle moteur ou de robotique :**

```rust 
use embedded_trig_f32::{sin, cos, tan, sincos, atan2, consts::PI};

fn main() {
    let angle = PI / 4.0; // 45 degrés
    
    // Calcul direct
    let s = sin(angle);
    let c = cos(angle);
    let (s2, c2) = sincos(angle);

    // Calculs supplémentaires
    let t = tan(angle).unwrap_or(0.0);
    
    // Calcul inverse sécurisé
    match atan2(s, c) {
        Ok(res) => {
            // res est proche de PI/4
        },
        Err(e) => {
            // Gestion de l'erreur (ex: NaN ou Undefined)
        }
    }
}
```

----

# 🧪 Précision & Tests

**La bibliothèque est testée avec une tolérance de 2.10^{-3} (Bhaskara).**

Pour lancer les tests sur votre machine de développement :

```
cargo test
```

----

# ⚖️ Licence

Ce programme est un logiciel libre ; vous pouvez le redistribuer et/ou le modifier selon les termes de la Licence Publique Générale GNU (GPL) version 2 ou ultérieure.

**Copyright (C) 2026 Jorge Andre Castro**