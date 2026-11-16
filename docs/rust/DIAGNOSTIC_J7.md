# Adaptateur du diagnostic technique J7

F-CFG-006 et v/l de F-CFG-005 relèvent de RS-PLATFORM-J7.
[Contrat portable](../agnostique/DIAGNOSTIC_J7.md)

`sns-settings::diagnostics` reçoit des événements énumérés et garde un journal
borné partagé entre lanceur et atelier. L’écriture JSONL utilise un seul worker.
La fermeture normale attend les écritures acceptées et signale une défaillance
d’écriture ou de synchronisation. Le chemin est annoncé séparément en console.

Le résolveur de destinations de `sns-project` est partagé par le lanceur et
l’atelier. Le journal de lancement réclame un fichier neuf avec `create_new`.
Le chemin du journal en cours est réservé comme le sont déjà les autres rôles
de fichier. L’atelier capture un instantané puis utilise le worker d’export de
documents pour l’écriture atomique. Le sélecteur natif fournit la destination.

```sh
cargo run -p sns-desktop --locked --offline -- --check-config -v -l
cargo test -p sns-desktop --test startup --locked --offline
cargo test -p sns-settings --lib --locked --offline
cargo test -p sns-ui-egui --test configuration --locked --offline
```

Dans `Main.java` de l’archive Filius, `-l` active la configuration Logback et son
fichier dans le dossier utilisateur. `FiliusArgs.java` analyse aussi `-v`, dont le
champ n’est pas consommé par le démarrage examiné. Sinesi donne un effet propre
à chaque option et utilise le répertoire de travail effectif pour le nom automatique.
