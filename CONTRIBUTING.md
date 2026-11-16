# Contribuer à Sinesi

Lire [AGENTS.md](AGENTS.md), [l’état de reprise](docs/suivi/ETAT.md) et les exigences liées à la tranche choisie. Les propositions de réalisation restent séparées des comportements portables.

Le [guide de développement](docs/rust/DEVELOPPEMENT.md) est la référence des
commandes de préparation, contrôle, CI et tests graphiques. Il explique aussi le
choix des tests, les frontières de crates et la lecture des preuves historiques.
Le [processus portable](docs/agnostique/EVOLUTION.md) définit les tranches et leurs
critères sans imposer un langage.

Après changement de suivi, régénérer avec `python3 scripts/docs.py render`. Une exigence est vérifiée seulement avec une preuve reproductible dans le registre. Les futures applications installables ne doivent pas être annoncées comme disponibles à partir d’un écran de prototype.

Pour une modification d’interface, comparer les références courantes et inspecter
les images. Leur remise à jour est une décision de revue; ne pas l’activer pour
faire disparaître une régression. Les anciennes captures de qualification restent
historiques et la dette de réconciliation de la suite complète reste visible.

Le corpus historique `_archive/` est facultatif et reste hors Git. `python3 scripts/docs.py check --require-reference` exige sa présence et contrôle ses empreintes; sans corpus, le contrôle structurel annonce cette limite. Ne pas reconstruire l’inventaire avec `audit` en l’absence de l’archive.
