# Contribuer à Sinesi

Lire [AGENTS.md](AGENTS.md), [l’état de reprise](docs/suivi/ETAT.md) et les exigences liées à la tranche choisie. Les propositions de réalisation restent séparées des comportements portables.

Le toolchain est fixé par `rust-toolchain.toml`. Vérifications du socle :

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python3 scripts/check_layers.py
python3 scripts/docs.py check
```

Après changement de suivi, régénérer avec `python3 scripts/docs.py render`. Une exigence est vérifiée seulement avec une preuve reproductible dans le registre. Les futures applications installables ne doivent pas être annoncées comme disponibles à partir d’un écran de prototype.

Les tests GPU de capture sont optionnels : `cargo test -p sns-ui-egui --features ui-snapshots --test workbench rendered_workbench_reference --locked`. La création/remise à jour des références est une opération de revue; ne pas activer la mise à jour automatique pour faire disparaître une régression.

Le corpus historique `_archive/` est facultatif et reste hors Git. `python3 scripts/docs.py check --require-reference` exige sa présence et contrôle ses empreintes; sans corpus, le contrôle structurel annonce cette limite. Ne pas reconstruire l’inventaire avec `audit` en l’absence de l’archive.
