# J7 — prototype session et récupération (historique)

Ce document conserve le premier prototype. L’intégration ultérieure et sa qualification figurent dans [J7-SESSION](J7-SESSION.md); les limites ci-dessous décrivent cette étape antérieure.

Passe du **8 novembre 2026**, macOS arm64, Rust 1.95.0. F-PROJ-003/006 et RS-PROJECTS-J7 restent **in_progress**. Le [profil portable](../agnostique/SESSION_J7.md) et la [décision technique](../rust/decisions/D-009-session-j7.md) distinguent le socle présent et les portes de sortie restantes.

Commandes réellement exécutées :

- `SNS_RECORD_J7_SESSION="$PWD/docs/verification" cargo test -p sns-project --test session --locked --offline` : **2 tests réussis**, 1 helper ignoré par défaut; le parent lance effectivement ce helper dans un autre processus et le tue après publication durable. [Journal](j7-session-backend-prototype.log).
- `cargo clippy -p sns-project --all-targets --locked --offline -- -D warnings` : réussi sans avertissement. [Journal](j7-session-backend-clippy.log).

Les tests couvrent exclusion des instances actives, deux identités distinctes, possession exclusive d’un candidat, refus transactionnel de vues/projets invalides, conservation d’un fichier corrompu, suppression explicite et récupération après arrêt forcé. L’artefact de processus et le document récupéré sont désormais régénérés par la recette intégrée décrite dans J7-SESSION; les journaux de ce prototype restent historiques. Le fichier projet initial reste identique octet pour octet.

Ce prototype n’est pas encore branché à la GUI. Il ne prouve ni une autosauvegarde périodique, ni une reprise au lancement, ni la restauration des brouillons ou des préférences complètes. Aucun parcours graphique, contrat formel final ou résultat Windows/Linux n’est revendiqué pour cette tranche. Les trois lots précédents restent qualifiés sur leurs sources archivées.
