# D-009 — Récupération atomique par instance

Date : 8 novembre 2026. Décision mise en œuvre pour F-PROJ-003/006; RS-PROJECTS-J7 reste ouvert pour l’import.

`sns-project::session` porte `SessionRecord`, `SessionView`, `SessionLease`, les résumés et le scan borné. La persistance réutilise `write_atomic` : temporaire sur le même volume, synchronisation, remplacement et synchronisation du répertoire sur Unix. `serde`, déjà fixé dans le workspace, devient une dépendance directe de ce crate et de la GUI; aucune version externe n’est modifiée.

Le verrou est un fichier compagnon, jamais l’inode remplacé par l’écriture atomique. `File::create_new` attribue un nom par instance; `File::try_lock` exclut les propriétaires coopérants tant que la session est active. L’API est disponible dans la toolchain 1.95 utilisée. Les règles suivent la [documentation standard](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock); les tests valident leur fonctionnement dans un autre processus sur macOS, sans annoncer une preuve Windows/Linux.

Un verrou `.allocation.lock` sérialise création et nettoyage. L’allocation réserve verrou et fichier vide avant de relâcher cette garde, évitant qu’un scan retire un verrou créé mais pas encore possédé. L’attente est bornée à 50 essais espacés de 5 ms dans le fil de stockage. Le nettoyage ne supprime que les réservations vides inactives; les documents non vides invalides sont conservés. Les limites 128/256/32 et les quotas du contenu figurent dans le [profil portable](../../agnostique/SESSION_J7.md).

`session_ui.rs` orchestre un seul travail de stockage à la fois, via fil nommé et canal borné de capacité 1. Le propriétaire du verrou voyage avec le travail puis revient dans son résultat. L’instantané `Arc<SessionRecord>` fige les données; son validateur d’édition est exécuté avant l’écriture. La reprise valide projet et brouillons avant de remplacer l’éditeur, renouvelle le worker et revient en conception. Les compteurs d’envoi mail ne sont pas persistés comme données de brouillon.

Le chemin natif est `LOCALAPPDATA/Sinesi/sessions` sur Windows (repli APPDATA), `~/Library/Application Support/Sinesi/sessions` sur macOS et `XDG_STATE_HOME/sinesi/sessions` ou `~/.local/state/sinesi/sessions` sur Linux. `SINESI_SESSION_DIR` permet un stockage explicitement isolé. `LanUi::with_storage` accepte les trois chemins de test; `native_with_project` utilise les chemins par défaut. Le lancement desktop accepte un fichier positionnel et la frontière `--`; le reste du CLI cible reste ouvert.

Le bouton de fermeture conservant la récupération force la publication sans attendre le cycle de deux secondes. La fermeture est différée jusqu’à l’acquittement. Les chemins de préférences, de récupération et du projet ouvert sont protégés contre les écritures accidentelles croisées. La récupération n’est pas une désérialisation de VM; les checkpoints restent distincts.

Deux recettes arrêtent réellement un producteur après publication durable : la première exerce le stockage, la seconde l’autosauvegarde de production dans le harnais graphique avec saisies non appliquées. Les helpers sont ignorés par défaut et lancés explicitement par leurs parents, avec répertoires isolés et gardes de terminaison. Le [rapport](../../verification/J7-SESSION.md) contient les commandes et artefacts. Le [prototype antérieur](../../verification/J7-SESSION-EN-COURS.md) est conservé comme étape historique.

## Complément pendant la tranche annotations

Une passe intégrale a montré un scan sans diagnostic pour un fichier corrompu dont le propriétaire venait d’être détruit, alors que le test attendait son diagnostic. Le fichier restait intact. Le verrou était donc temporairement vu comme actif. Ce résultat est compatible avec une description de fichier héritée pendant la création concurrente du processus enfant; cette attribution précise reste une inférence, sans instrumentation du noyau.

La [documentation Apple de flock](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html) précise le partage des références de verrou après duplication/fork. `LockedFile` appelle désormais explicitement `File::unlock` dans `Drop`, avant de fermer le descripteur, pour les sessions et la garde d’allocation. Le propriétaire n’est jamais cloné et reste déplacé entre le fil et la GUI. Un autre processus réellement actif reste exclu; l’arrêt forcé repose toujours sur la libération par l’OS. La nouvelle règle ne rend pas un fichier invalide récupérable et ne supprime pas ses octets.

La recette de stockage est répétée **40 fois**, chaque fois avec trois tests normaux et un enfant réellement arrêté; les 40 répétitions passent. Le [rapport annotations](../../verification/J7-ANNOTATIONS.md) conserve l’échec initial, cette passe de répétition et la qualification finale. La preuve reste locale à macOS; elle ne remplace pas les validations trois OS.
