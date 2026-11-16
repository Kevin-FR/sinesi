# J8 — interface élève : niveaux, espaces, palette, câble, trajet et terminal

Tranches F-UX-001 à F-UX-006 / RS-UX-J8. Le [comportement portable](../agnostique/EXPERIENCE_ELEVE.md)
et [D-015](../rust/decisions/D-015-interface-eleve.md) fixent la portée. RS-UX-J8
reste ouvert pour les missions évaluables (F-ADV-004).

## Résultat

- L’atelier démarre au niveau **Élève**. Import Filius, exécutions et export du
  réseau n’y sont pas proposés; le niveau **Expert** les révèle sans changer le
  projet ni l’espace courant (R-UX-001).
- Le niveau est enregistré dans le profil d’affichage et relu au démarrage. Le
  profil v2 existant, sans `level`, est accepté sans diagnostic ni réécriture et
  vaut Élève (R-UX-002). Le résolveur exigeait jusque-là tous les champs d’un profil
  complet : la régression a été détectée par le test et corrigée.
- Construire, Tester et Annoter remplacent les libellés Conception, Simulation et
  Documentation, au clic comme par F2/F3/F4, avec une consigne colorée dans les six
  langues (R-UX-003). Au niveau Élève, la ligne d’exécution devient « Simulation :
  En pause/En cours »; l’identifiant et les compteurs restent au niveau Expert.
- 36 nouvelles références `j8-student-*` couvrent 6 langues × Icônes/Schéma × 3
  espaces (R-UX-004).

## Références graphiques

Les 519 PNG J0–J7 sont archivés dans
`crates/sns-ui-egui/tests/snapshots-historique/atelier-j7/` avec `MANIFEST.sha256`
(`shasum -a 256 -c MANIFEST.sha256` passe). Après régénération : 315 références
j6/j7 changent avec la barre, 204 sont identiques (dont les captures j0–j5 que les
tests actuels ne produisent plus), 36 sont nouvelles.

Écart antérieur absorbé : `j6-mail-client-*` échouait déjà avant cette tranche
(images de diagnostic du 7 novembre). La fenêtre de messagerie s’ouvre plus étroite
qu’en référence (760 au lieu de 870 points) et la zone de texte est rognée
horizontalement. Corrigé ensuite dans la tranche des missions
([preuves](J8-MISSIONS.md)) : le contenu suit la largeur de la fenêtre.

## Commandes exécutées

Environnement : macOS 27.2 arm64, Apple M2 Max, Rust 1.95.0, egui 0.37.0 et wgpu.
Empreinte des sources de la tranche :
`sha256:40d89a351ee3489a6424f37c40b513a0f1cc13d54d7afe7fac2c24bd67f1c108`.

| Commande | Résultat |
| --- | --- |
| `cargo xtask ui` avant modification (base) | 25 tests, pass |
| `cargo test -p sns-ui-egui --locked --offline` avant modification (base) | pass |
| `cargo xtask check` | pass — 184 exigences, 51 tâches |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | pass |
| `cargo test --workspace --locked --offline` | pass |
| `cargo test -p sns-settings --test display --locked --offline` | 5 tests, pass |
| `cargo test -p sns-ui-egui --features ui-snapshots --locked --offline --no-fail-fast` après régénération | pass |
| `cargo xtask ui` (suite `student` ajoutée) | 28 tests, pass |

Tests : `crates/sns-ui-egui/tests/student.rs`,
`crates/sns-settings/tests/display.rs`. Les tests existants ont été adaptés aux
nouveaux libellés et passent au niveau Expert lorsqu’ils exercent une commande
avancée (option temporaire de ligne de commande pour le diagnostic, afin que le
profil reste inchangé).

## Tranche 2 — palette illustrée et câble guidé (F-UX-003)

- La palette affiche six tuiles colorées avec le symbole de la vue courante. Leur
  nom accessible reste « Ajouter un … », donc les tests et le clavier existants
  sont inchangés (Tab puis Entrée).
- L’outil Câble devient un bouton à bascule pleine largeur. Une bulle sur le
  canevas guide les deux clics, nomme l’équipement de départ, puis confirme
  « Câble posé entre A et B ! ». La bulle est un vrai libellé, lisible par les
  technologies d’assistance.
- Test `illustrated_palette_and_guided_cable_connect_a_new_computer_in_six_languages`
  (R-UX-005) et 12 références `j8-cable-*`. 23 références existantes changent
  uniquement à cause de la palette (écarts d’image seulement, aucune assertion en échec).

Commandes : `cargo xtask check`, Clippy workspace, `cargo test --workspace`,
suite graphique complète sans régénération et `cargo xtask ui` — toutes en succès.

## Tranche 3 — paquets animés et récit du trajet (F-UX-004)

- `src/story.rs` dérive des phrases de la trace canonique (fonction pure, lecture
  seule, parcours depuis la fin limité à 40 étapes). `workbench/packets.rs` rejoue
  les faits `ethernet.sent` comme pastilles sur les câbles; `workbench/story_ui.rs`
  affiche le panneau sous le canevas en mode Tester, y compris en disposition à onglets.
- Les traces de référence ont été relevées avec la CLI sur `fixtures/valid/routed.json` :
  avec passerelle, ARP, deux transferts routés, réception puis réponse en 12 ms;
  sans passerelle, `icmp.echo.requested` puis `icmp.error` de raison `routing.no_route`.
- Test `packet_journey_is_animated_and_told_from_the_trace_including_a_missing_gateway`
  (R-UX-006) dans 6 langues × 2 vues : paquets en vol pendant le pas à pas, phrases
  attendues, trace identique avant/après l’affichage, explication de l’absence de
  passerelle. Références `j8-story-success-*`, `j8-story-no-gateway-*`, `j8-packet-*`.
- Le harnais avance de 0,25 s par pas; la mesure des paquets en vol et la capture
  intermédiaire se font donc avant de laisser l’interface au repos.

## Tranche 4 — terminal lisible (F-UX-005)

- `workbench/desktop/terminal_text.rs` lit le dernier mot de commande et la forme du
  résultat; le moteur et le format des lignes du terminal ne changent pas.
- Test `terminal_results_read_as_sentences_for_students_and_stay_structured_for_experts`
  (R-UX-007) sur `fixtures/valid/hosts.json`, six langues : `ipconfig`, `arp` vide,
  `ping -c 1`, `arp` rempli, puis bascule Expert dans la même session.
  Références `j8-terminal-student-*` et `j8-terminal-expert-*`.
- Trois tests unitaires pour `route` (boucle locale omise, passerelle par défaut),
  `ls`, les formes inconnues et les messages de ping.
- Deux erreurs de mon test corrigées pendant la mise au point : avancer après la fin
  du ping laissait expirer légitimement le cache ARP (60 s simulées), et l’adresse MAC
  de Portable B est désormais lue dans le projet.

## Tranche 5 — fiche pédagogique (F-UX-006)

- `sns-topology::gateway_advice` juge la passerelle d’un hôte à adresse fixe : vide,
  invalide, adresse propre, hors réseau (avec le réseau), joignable. Test
  `crates/sns-topology/tests/advice.rs`, y compris /16, DHCP et routeur.
- L’inspecteur affiche les aides en Construire au niveau Élève et l’avis aux deux
  niveaux. Test `equipment_sheet_explains_fields_and_judges_the_gateway_while_typing`
  (six langues), références `j8-sheet-*`. Les aides en petite police ont été
  agrandies après inspection : elles étaient presque illisibles.
- Le test historique du ping au clavier fait défiler l’inspecteur, devenu plus long
  avec l’avis, avant d’activer « Envoyer un ping ».

## Limites

Preuves locales macOS uniquement, par le harnais egui; aucune manipulation native
au clavier ni qualification Windows/Linux. F-UX-001 à F-UX-006 sont vérifiées localement;
la [recette native macOS](J8-NATIF-MACOS.md) complète le harnais. Linux et Windows : [trois systèmes](J8-TROIS-OS.md).
