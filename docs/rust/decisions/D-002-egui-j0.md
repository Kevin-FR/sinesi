# D-002 — Adaptateur egui J0

Direction réalisée : Rust 1.95.0, egui/eframe/egui_extras/egui_kittest **0.37.0 upstream**, avec versions transitives verrouillées dans `Cargo.lock`. Les fonctionnalités `default_fonts` et `monochrome_emoji_fonts` sont actives dans les vues de production et de test. L’application eframe utilise wgpu, AccessKit et le presse-papiers; les backends X11/Wayland sont déclarés pour Linux.

`sns-desktop` assemble la fenêtre et `sns-ui-egui` dessine des projections. Un worker dédié possède le moteur; les commandes passent par une file bornée de huit travaux. Le worker retourne un instantané et demande un repaint. Les frames ne font jamais avancer le temps de simulation. Une attente synchrone existe uniquement pour les tests appelant explicitement le worker.

Le prototype présente un canevas avec zoom, déplacement et sélection, une table virtuelle à colonnes redimensionnables et un navigateur pédagogique. La charge de 10 000 lignes est produite par 5 000 vrais envois de prototype, pas par une table de données fictives indépendante du moteur.

La pile upstream suffit à cette première tranche. Aucun fork egui/winit et aucune brique craft-libs n’est activé. Leur étude reste utile pour le docking et le fenêtrage ultérieurs; l’adoption devra démontrer un besoin et une compatibilité de sources. Les crates rfd, tiny-skia et craft-pdf restent également hors du socle J0 : leurs fonctions appartiennent aux livraisons correspondantes.

Les tests egui_kittest interrogent les widgets réels, activent des contrôles, sélectionnent un équipement par focus/Entrée, chargent une page/image et soumettent un formulaire. Le test de capture est optionnel via `ui-snapshots` et utilise le backend wgpu. Les captures de référence sont revues avant d’être acceptées. Ces tests ne remplacent pas la validation native multi-écran/IME sur chaque OS.

Plateforme locale de réalisation : macOS arm64. La CI configure Windows, Linux et macOS; son ajout ne constitue pas une preuve que ses jobs distants ont été exécutés.
