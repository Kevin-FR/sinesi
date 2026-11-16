# Polices de document embarquées

Noto Sans et Noto Sans Mono, variantes Regular et Bold, sont copiées sans modification depuis le [dépôt officiel Noto](https://github.com/notofonts/noto-fonts/tree/ffebf8c1ee449e544955a7e813c54f9b73848eac), révision `ffebf8c1ee449e544955a7e813c54f9b73848eac`. La [licence OFL-1.1](OFL.txt) et son copyright sont redistribués avec les fontes. Les sources exactes, tailles et SHA-256 figurent dans [provenance.json](provenance.json).

Les fichiers sont embarqués dans le binaire via `sns-fonts`; aucun téléchargement ni installation de police n’est nécessaire à l’exécution. Ils servent aux annotations et aux futurs adaptateurs d’export; ils ne remplacent pas les polices des menus egui. `python3 scripts/check_fonts.py` contrôle hors ligne empreintes, poids Regular/Bold et glyphes du corpus FR EN ES IT PT DE. Ce contrôle ne prétend pas couvrir tous les scripts Unicode.
