# J7 — rapport PDF qualifié localement

**F-DOC-004 est vérifié; RS-REPORTING-J7 est vérifié pour ses quatre exigences
F-DOC-001 à 004.** Le volet documentaire de R-RAPPORT est couvert; sa validation
plateforme/configuration/accessibilité reste ouverte dans les autres lots J7.
Le [profil portable](../agnostique/RAPPORTS_J7.md) et
[D-012](../rust/decisions/D-012-rapport-pdf-j7.md) décrivent comportement et adaptation.

## Réalisation

`sns-report` capture les données appliquées, compose les sections et pagine les
tables, puis produit un PDF avec Krilla 0.8.2. La scène partagée conserve formes,
texte, annotations et styles; les symboles viennent des mêmes SVG, incorporés en
images 256 × 256. Quatre fontes sont embarquées avec correspondance Unicode; les
textes de figure et de tables sont sélectionnables. Aucun fichier système de fonte,
ressource externe, action automatique, formulaire ou code actif n’est requis.

La GUI propose PDF dans « Exporter le réseau… »; sélection native, génération et
écriture partagent le worker documentaire borné de PNG. La destination est protégée
et réservée; une fermeture normale attend le travail. Le CLI expose `report-pdf`
sur projet ou checkpoint validé, sans dialogue. Les deux utilisent le même générateur.

Le rapport sépare configuration durable, état effectif, routes courantes et trafic
capturé. L’adresse obtenue par DHCP ne remplace pas la configuration DHCP. Les
routes du client sont projetées depuis l’équipement effectif. Le regroupement MAC
conserve toutes les émissions/réceptions canoniques et les identités enregistrées,
y compris après changement de génération. Les historiques incomplets sont annoncés.
Les mots de passe fictifs du scénario restent absents; aucun corps mail n’est exporté.

## Qualification exécutée

Environnement : macOS arm64, Rust 1.95.0, egui 0.37.0/wgpu. Voir
[l’environnement complet](j7-report-environment.json) et
[les nouvelles dépendances et licences](j7-report-dependencies.json).

| Commande réellement exécutée | Résultat / preuve |
| --- | --- |
| `SNS_RECORD_J7_REPORT="$PWD/docs/verification" cargo xtask ci` | **233 tests du workspace plus un enfant du pont**, cinq ignorés; Clippy sans avertissement, formatage et contrôles communs réussis; les quatre preuves PNG sont toujours décodées. [Journal](j7-report-ci.log) |
| `cargo xtask ui` | **6 tests GPU**, **24 nouvelles captures comparées sans mise à jour**, PNG/PDF, six langues et deux vues. [Journal](j7-report-gpu.log) |
| `SNS_RECORD_J7_REPORT="$PWD/docs/verification" cargo test -p sns-report --locked --offline` | Trois tests : scénario routé/DNS/DHCP localisé et déterministe, pagination de 1 200 paquets, erreurs/quota avant écriture; également inclus dans `ci` |
| `python scripts/check_pdf_report.py PDF --metadata JSON`, pour chacun des 13 rapports ci-dessous | **13 réussites** avec pypdf 6.10.0 : pages A4, sections, empreinte, compte exact de paquets, quatre polices embarquées et Unicode, mots de passe absents. [Journal](j7-report-independent.log) |
| Poppler `pdftoppm -scale-to 1000 -png PDF PREFIX`, plus pages détaillées | Toutes les pages FR/Schéma inspectées, aperçu DE/Icônes et dernières pages du stress; captures GUI FR/PDF et variantes courantes comparées. [Planche de revue](j7-report-review-fr.png) |
| Lecture de géométrie par pdfplumber 0.11.9 | Zéro caractère hors des marges déclarées sur FR/Schéma, DE/Icônes et IT/Schéma. [Résultats](j7-report-layout.json) |
| `python3 scripts/docs.py render`, puis `python3 scripts/docs.py check --require-reference` | Vues, registres, liens et archive de référence vérifiés. [Journal](j7-report-docs.log) |
| `python3 scripts/check_manifest.py check docs/verification/j7-report-code-manifest.json` | Inventaire courant identique à la révision scellée |

Les tests des adaptateurs sont `sns-cli/tests/report.rs` et
`sns-ui-egui/tests/report.rs`. Ils couvrent déterminisme, fichier source protégé,
destination préservée sur erreur, capture en simulation sans commande/avancement,
exclusion des brouillons, interaction des contrôles, refus d’une seconde opération
et réservation contre une sauvegarde. Les tests PNG existants qualifient le cycle
partagé et conservent leur comparaison octet pour octet avec le backend.

Les **321 images UI antérieures** restent identiques. Les nouvelles références
`j7-report-{png,pdf}-*` décrivent l’ajout légitime des contrôles PDF, sans remplacer
les preuves historiques. Deux processus de récupération sont réellement arrêtés
dans la suite globale; aucun nouveau stress de récupération n’est revendiqué.
Les corpus d’import et la mesure historique restent ignorés par défaut ici.

## Artefacts de référence

Le scénario [report-network.json](../../fixtures/valid/report-network.json)
contient deux routeurs, DNS, un client DHCP, un client mail à mot de passe fictif,
notes stylées et ping. Les douze rapports FR EN ES IT PT DE × Icônes/Schéma
contiennent chacun **34 paquets**, six groupes MAC et **8 ou 9 pages** suivant la
langue. Chaque PDF a des métadonnées JSON adjacentes avec son empreinte et les
textes exigés.

- [FR/Schéma](j7-report-fr-diagram.pdf), [FR/Icônes](j7-report-fr-icons.pdf).
- [EN/Schéma](j7-report-en-diagram.pdf), [ES/Schéma](j7-report-es-diagram.pdf),
  [IT/Schéma](j7-report-it-diagram.pdf), [PT/Schéma](j7-report-pt-diagram.pdf),
  [DE/Icônes](j7-report-de-icons.pdf); les variantes complémentaires sont dans le même dossier.
- [Pagination](j7-report-pagination.pdf) : **1 200 lignes sur 72 pages**, première
  et dernière lignes présentes, en-têtes et contexte MAC répétés, trace partielle annoncée.

## Correction de saisie non représentable

Un caractère U+10FFFF valide dans le document provoquait un panic du canevas egui
avant de pouvoir afficher l’erreur d’export. Le [journal initial](j7-report-glyph-failure.log)
est conservé. Le canevas utilise désormais explicitement le remplacement visuel
pour permettre la correction; le texte original n’est pas modifié. Le générateur
refuse ce glyphe avant écriture et conserve le fichier existant. La régression
reproduit ce chemin dans le harnais de l’application et passe dans les suites finales.

## Sources et limites

[Manifeste](j7-report-code-manifest.json) : **618 fichiers**, révision
`sha256:d7a7595b59ad42103a434735ed6ba68f88032f75570c97c6b2633fe2216cb66e`.
Le [delta de sources](j7-report-source-delta.json) s’applique à l’archive initiale
d’import, avec les suppressions indiquées, puis se vérifie par le manifeste.
Aucun paquet tiers existant n’est retiré ou remplacé; douze versions supplémentaires
et la crate locale de rapport rejoignent le lockfile. Les archives de référence
restent en lecture seule et ne sont pas redistribuées.

La présence du PDF ne qualifie pas PDF/A ou PDF/UA, la fidélité de tous les scripts
Unicode, la lecture/édition de PDF ou les symboles entièrement vectoriels. Les
quotas documentaires sont explicites; ils ne constituent pas un budget de mémoire
mesuré pour toute l’application. Le sélecteur natif n’a pas été manipulé manuellement.
Aucun résultat Windows/Linux ou CI distante n’est revendiqué. La suite GPU historique
complète, la lisibilité globale, la configuration et la livraison restent ouvertes.
J7 n’est pas terminé : l’import est partiel et le lot plateforme reste à réaliser.
