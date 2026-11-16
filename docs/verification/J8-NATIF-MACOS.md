# J8 — parcours élève dans l’application native macOS

Recette native de F-UX-001 à F-UX-006 et F-ADV-004 (RS-UX-J8), complémentaire des
preuves par harnais de [J8-UX-NIVEAUX](J8-UX-NIVEAUX.md) et [J8-MISSIONS](J8-MISSIONS.md).

## Environnement

macOS 27.2 arm64 (Apple M2 Max), Rust 1.95.0, binaire `target/debug/sns-desktop`
construit par `cargo build -p sns-desktop --locked --offline` au commit `d4e70cd`
plus le chargement des missions d’enseignant. Lancement avec un profil temporaire :
`SINESI_SETTINGS_PATH=<tmp>/settings.json target/debug/sns-desktop --session-dir <tmp>/session`.
Pilotage par l’arbre d’accessibilité macOS (AX) de la fenêtre native.

## Parcours exécuté

| Étape | Observé dans la fenêtre native |
| --- | --- |
| Démarrage | Niveau Élève actif; Construire/Tester/Annoter, bouton Missions; commandes Expert absentes; palette illustrée; consigne de Construire. |
| Missions | Carte : chapitres 1 et 2, étoiles vides, « Le câble oublié » ouvert, missions suivantes verrouillées, bouton « Ouvrir une mission de l’enseignant… ». |
| Commencer | Projet de la mission installé; carte de mission avec deux objectifs « À faire ». |
| Vérifier | Explication affichée : « Poste C n’a aucun câble : aucun paquet ne peut lui parvenir. » |
| Outil câble | Bulle « Clique sur le premier équipement à relier. », puis « Câble posé entre Poste C et Commutateur ! »; objectif « Réussi », explication retirée. |
| Tester | Panneau « Le voyage du paquet »; fiche de Poste A avec l’avis « Vide : cet appareil ne pourra parler qu’à son propre réseau. » |
| Ping 192.168.1.4, Reprendre | Récit jusqu’à « Poste A reçoit la réponse de 192.168.1.4 en 8 ms : la connexion fonctionne ! »; « Mission réussie ! », trois étoiles, « Mission suivante ». [Capture](j8-native-macos-mission.png) |
| Expert | « Importer Filius… », « Exécutions » et « Exporter le réseau… » apparaissent. |
| Profil | `display.level = expert`, `missions = {"forgotten-cable": 3}` relus dans le fichier écrit par l’application. |

## Limites

La fenêtre native a été manipulée sous macOS; Linux et Windows sont couverts par
le harnais sur chaque système ([trois systèmes](J8-TROIS-OS.md)).
