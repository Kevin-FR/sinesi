# Sinesi — Simple network simulator

Sinesi est une application opensource de simulation réseau pédagogique, écrite en Rust. Le socle J0 contient un moteur déterministe sans interface, un lanceur CLI, des contrats JSON versionnés et un atelier natif egui/eframe 0.37.

**État : prototypes de fondation.** Le canevas, la trace et les échanges applicatifs de démonstration fonctionnent; Ethernet, ARP, TCP/IP, DHCP et DNS ne sont pas encore implémentés. Les événements sont nommés `prototype.message.*` pour rendre cette limite visible.

## Lancer le socle

Le toolchain Rust 1.95.0 est défini dans `rust-toolchain.toml` et les dépendances dans `Cargo.lock`.

```sh
# Atelier graphique : canevas, échanges, navigateur pédagogique
cargo run -p sns-desktop --locked

# Même moteur sans dépendance graphique ni fenêtre
cargo run -p sns-cli --locked -- run fixtures/valid/scenario.json --seed 42 --trace /tmp/sinesi-trace.jsonl
cargo run -p sns-cli --locked -- run fixtures/valid/scenario.json --until-us 999
cargo run -p sns-cli --locked -- validate fixtures/valid/project.json
```

Après un premier téléchargement des dépendances, ces commandes acceptent aussi `--offline` placé avant `--`. Le CLI retourne 0 pour une exécution terminée ou arrêtée à la borne virtuelle demandée, 1 pour une erreur et 2 quand le budget d’événements est épuisé. Le JSON de résultat et le fichier de trace permettent de distinguer ces situations.

Le navigateur du prototype charge une page, une image et des formulaires via le service événementiel embarqué. Il n’ouvre aucun serveur ou navigateur réel. Les fichiers choisis sur la ligne de commande restent des entrées explicites de l’adaptateur CLI.

## Vérifier et contribuer

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python3 scripts/check_layers.py
python3 scripts/docs.py check
```

Voir [CONTRIBUTING.md](CONTRIBUTING.md), les [contrats implémentés](contracts/README.md) et le [rapport J0](docs/verification/J0.md). La CI est configurée pour Windows/Linux/macOS; les résultats réellement obtenus sont recensés dans le rapport.

## Cahier des charges et suivi

Commencer par le [guide du dossier](docs/README.md), puis lire le [cahier des charges](docs/agnostique/CAHIER_DES_CHARGES.md) et la [roadmap](docs/agnostique/ROADMAP.md).

| Référence | Contenu |
| --- | --- |
| [Catalogue fonctionnel](docs/agnostique/CATALOGUE.md) | 177 exigences, critères de recette et preuves de code |
| [Architecture agnostique](docs/agnostique/ARCHITECTURE.md) | Découpage du produit et principes de simulation |
| [Matrice de fidélité](docs/agnostique/FIDELITE.md) | Capacités démontrées, simplifications et protocoles futurs |
| [Contrats portables](docs/agnostique/CONTRATS.md) | Modèle, commandes, événements, fichiers et erreurs |
| [Volet Rust](docs/rust/ARCHITECTURE.md) | Architecture native egui/eframe, cœur indépendant et compromis |
| [Pile et dépendances Rust](docs/rust/DEPENDANCES.md) | Bibliothèques, briques Craft, forks et étapes de validation |
| [Plan Rust](docs/rust/PLAN.md) | 49 lots reliés aux exigences |
| [État et reprise](docs/suivi/ETAT.md) | Situation réelle et prochaine tranche de travail |
| [Matrice de traçabilité](docs/suivi/MATRICE.md) | Exigence ↔ jalon ↔ tâche Rust |
| [Audit de référence](docs/audit/METHODE.md) | Périmètre, méthode, limites et divergences observées |

Les fichiers `docs/agnostique/` constituent la spécification réutilisable dans un autre langage. Les détails Rust sont confinés à `docs/rust/`. Les identifiants `F-*` relient les deux; les registres JSON de `docs/suivi/` permettent un suivi exploitable par un agent.


## Licence et référence historique

Le code neuf de Sinesi est sous [licence MIT](LICENSE). L’inventaire est fondé sur l’archive locale Filius 2.14.0; ce corpus, ses actifs et son runtime ne sont pas embarqués dans le dépôt ou le binaire Sinesi. Les références et empreintes restent dans `docs/audit/`. Voir la [décision de provenance](docs/rust/decisions/D-007-provenance.md).

Sans l’archive locale, le contrôle documentaire vérifie la structure du suivi et annonce qu’il ne revalide pas le contenu historique. Pour un contrôle complet : `python3 scripts/docs.py check --require-reference`.
