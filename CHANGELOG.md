# Changelog

## [0.3.0] - 2026-10-11

### Ajouts
- Ajout de `atan(x)` (infaillible : `atan(±∞) = ±π/2`, `atan(NaN) = NaN`).
- Dépendance à [`embedded-f32-sqrt`](https://crates.io/crates/embedded-f32-sqrt) `0.2` pour la racine carrée.
- Constantes `consts::FRAC_PI_4` et `consts::FRAC_2_PI`.
- Tests par balayage dense contre `f64` (sin, cos, atan, asin, acos, quatre quadrants, ±0, grands arguments).

### Changements
- **Précision** : l'approximation de Bhaskara I (erreur ~1,6·10⁻³) est remplacée par une réduction d'argument par quadrant (Cody-Waite) et des polynômes minimax, pour une erreur de l'ordre de 10⁻⁷.
- `sincos` ne fait plus qu'une seule réduction d'argument ; `sin` et `cos` ont chacun un chemin direct (fini `cos(x) = sin(x + π/2)`).
- `atan2` utilise un polynôme minimax avec réduction à `[0, tan(π/8)]` et gère `±0.0` via `is_sign_negative()`.
- `asin` utilise `(1-x)(1+x)` au lieu de `1-x²` ; `acos` utilise `atan2(sqrt(1-x²), x)` au lieu de `π/2 - asin(x)` (plus d'annulation catastrophique près de ±1).
- `tan` / `cot` détectent les pôles avec un seuil (`1e-7`) au lieu de `== 0.0` : `tan(PI/2)` renvoie désormais `Err(Undefined)`.
- `asin` / `acos` renvoient explicitement `Err(NonFiniteValue)` pour NaN / ±∞.
- Les arguments avec `|x| > 8192` sont d'abord réduits par `x % TAU` (résultat borné, précision réduite).
- `TrigError` est marqué `#[non_exhaustive]`.
- `#![deny(missing_docs)]` remplace `#![warn(missing_docs)]`.
- `#[inline]` sur toute l'API publique.

### Retraits
- Suppression de la branche `x.sqrt()` (cible ARM + `vfp2`) : `f32::sqrt` n'existe pas dans `core`. Le README n'annonce donc plus d'usage de l'instruction `VSQRT` ni de flags `target-feature`.
- La bibliothèque n'est plus « zéro dépendance » : elle dépend de `embedded-f32-sqrt`.

### Documentation
- README réécrit : précision réelle, comportement aux limites, suppression des mentions de Bhaskara / McCutcheon et de la tolérance 2·10⁻³.

## [0.2.0] - 2026-07-20

### Ajouts
- Ajout de la fonction `tan(x)` avec gestion d’erreurs pour les valeurs non finies et les singularités.
- Ajout de la fonction `cot(x)` pour le calcul de la cotangente avec gestion d’erreurs.
- Ajout de la fonction `sincos(x)` pour obtenir simultanément sinus et cosinus.
- Amélioration de la documentation publique avec un exemple d’utilisation plus complet.
- Mise à jour des métadonnées du crate pour refléter la nouvelle API.

### Améliorations
- Documentation du README enrichie pour couvrir les nouvelles fonctions.
- Meilleure visibilité des capacités de la bibliothèque pour les applications embarquées `no_std`.