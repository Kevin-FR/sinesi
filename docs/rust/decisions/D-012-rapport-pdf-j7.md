# D-012 — rapport PDF depuis un instantané

Décision du 8 novembre 2026. F-DOC-004 et RS-REPORTING-J7, après F-TOP-001,
F-OBS-001 et les exports PNG qualifiés. Aucun changement de format de projet.

## Choix

Ajouter `sns-report`, sans GUI ni I/O hôte, pour transformer un instantané en
rapport paginé puis en octets PDF. Il consomme la scène `sns-render`, les glyphes
`sns-fonts`, les données de modèle et les projections du moteur. Le CLI et le
workbench composent les mêmes fonctions; eux seuls ouvrent/écrivent les fichiers.

Krilla **0.8.2**, `default-features = false`, fonctionnalité `raster-images`, est
retenu directement après examen de son API et compilation. Il accepte les glyphes
déjà façonnés, sous-ensemble les fontes et écrit leur correspondance Unicode.
MSRV déclaré 1.92, compatible avec le toolchain 1.95 du dépôt. Sources primaires :
[documentation de la version](https://docs.rs/krilla/0.8.2/krilla/),
[glyphes](https://docs.rs/krilla/0.8.2/krilla/text/struct.KrillaGlyph.html) et
[surface](https://docs.rs/krilla/0.8.2/krilla/surface/struct.Surface.html).
Les sources du paquet verrouillé ont aussi été lues pour la normalisation des
avances et offsets, l’alpha des images et la sérialisation des métadonnées.

craft-pdf/Krilla était une proposition. L’adaptateur supérieur et son patch ne
sont pas nécessaires à ce chemin d’écriture; ni fork egui/winit, ni fonction de
lecture PDF, ni deuxième shaper ne sont ajoutés au produit. Les fonctionnalités
`simple-text`, `pdf` et `rayon` de Krilla restent désactivées. Le code ne fournit
aucune image arbitraire à ses décodeurs : seulement les pixels issus de nos SVG.

Krilla apporte une version interne Skrifa 0.42.1 et tiny-skia-path 0.12.0, distinctes
du shaping SNS 0.44.0 et du raster PNG 0.11.4. Seuls données de fonte et identifiants
de glyphes traversent cette frontière, pas leurs types internes. Le lockfile ajoute
12 paquets tiers et `sns-report`, sans retirer ou remplacer les versions précédentes.
Leurs licences déclarées sont contrôlées dans le graphe de la tranche.

## Réalisation

- `content.rs` : capture commune depuis l’Engine, distinction configuré/effectif,
  sections et tables, validation de trace, regroupement par MAC, pagination bornée.
  Les routes sont projetées depuis les équipements effectifs, notamment DHCP.
- `pdf.rs` : polices embarquées, glyphes normalisés avec clusters Unicode, titres,
  filets et pieds de page. La recherche de fin de cluster est préparée par ligne;
  elle ne rescane pas tous les glyphes pour chaque glyphe.
- `scene.rs` : rectangles/traits/texte vectoriels avec découpage, italique et
  soulignement; symboles SVG embarqués rendus en RGBA 256 × 256 puis incorporés.
  Ce rendu hybride conserve les quatre faces et ne prétend pas que les icônes
  sont vectorielles. Les images sont démultipliées avant incorporation.
- `Snapshot::capture` lit sans commande, pas de simulation ou socket. La restauration
  d’un checkpoint vérifie son historique mais ne poursuit pas son exécution.
- `export_ui.rs` : même tâche/canal bornés pour PNG et PDF; destination native
  asynchrone, instantané, réservation des chemins et attente à la fermeture.
  Les méthodes de cycle commun s’appellent désormais `document_export_*`, sans alias.
- `report_cli.rs` : `report-pdf INPUT --output REPORT [--input project|checkpoint]
  [--view icons|diagram] [--language fr|en|es|it|pt|de]`, sans dialogue.

La phase de lecture ne lit pas de polices système, images externes ou ressources
réseau. Le résultat est préparé avant `atomic_write_runtime`. Les identifiants de
run passés restent visibles après changement de génération; ils ne sont pas
réécrits pour correspondre au nouveau run d’un checkpoint restauré.

## Vérification indépendante et limites

Le lecteur `scripts/check_pdf_report.py` utilise **pypdf 6.10.0**, uniquement pour
la qualification, avec `requirements-pdf-verification.txt`. Il contrôle compte de
pages, sections, polices, Unicode, nombre de lignes et intégrité. L’application ne
dépend pas de Python. Poppler rend les pages; l’inspection visuelle complète les
assertions de texte. PDF/A, PDF/UA, signature, formulaires, chiffrement et lecteur
PDF intégré ne sont pas revendiqués.

Le test de glyphe manquant a révélé un panic du canevas egui 0.37. L’installation
des fontes choisit explicitement `MissingGlyphPolicy::Tofu` pour que l’utilisateur
puisse réparer la saisie. Le texte original reste intact et les exports refusent
ce caractère avant toute écriture. Le test graphique reproduit ce chemin.

L’ajout des contrôles PDF modifie la fenêtre d’export : les nouvelles références
PNG/PDF portent `j7-report-*`; toutes les références antérieures sont conservées.
Les preuves natives et trois OS restent une porte distincte de J7. Voir le
[rapport de qualification](../../verification/J7-RAPPORT.md).
