# Adaptateur des attentes de réponse — J7

F-CFG-005 / RS-PLATFORM-J7; non-régression RS-ENGINE-J7, F-SIM-007/008. Le
[contrat portable](../agnostique/ATTENTES_J7.md) fixe le comportement;
[D-014](decisions/D-014-attentes-j7.md) motive les choix.

## Propriétaires

- `sns-model::WaitFactor` borne 1–5, multiplie avec contrôle de débordement et
  sérialise un nombre. Le décodage accepte aussi les notations JSON intégrales
  telles que `3.0`, sans accepter texte, fraction ou valeur hors bornes.
- `sns-protocols::Network` possède la politique de l’exécution. Son contexte
  distingue les attentes des expirations; les timers TCP portent un indicateur
  transitoire pour séparer retransmission/fermeture de TIME_WAIT. Les réservations
  d’offres DHCP étendent simultanément leur échéance et leur timer de nettoyage.
- `sns-apps::TimerKind::waits_for_response` classe exhaustivement les attentes;
  une nouvelle variante impose une décision. `ApplicationPlan` applique la politique
  avant de valider et planifier les effets. Les cadences ne sont pas multipliées.
- `Engine::new_with_wait_factor` et `from_project_with_wait_factor` initialisent la
  politique avant les effets de démarrage. Les constructeurs ordinaires utilisent 1.
  Arrêt, redémarrage et remplacement préservent la politique; le remplacement
  explicite permet au workbench de reprendre celle du lancement pour un nouveau réseau.
- Le workbench conserve séparément la demande du lancement et le facteur effectif
  de son worker. Charger un scénario utilise la demande; restaurer ou rejouer laisse
  le fichier décider. La projection expose la valeur effective au rapport natif.

## Lancement

Le binaire desktop accepte `-r N` et `--rtt-factor N`. Son aide est localisée dans
les six langues. Une valeur invalide produit un diagnostic localisé sans recopier
la valeur rejetée et utilise 1. Les occurrences successives suivent la politique
du lanceur : la dernière valeur est appliquée ou remplacée par le repli sûr.
Cette option n’est pas exposée au prototype J0 du desktop.

Le CLI sans fenêtre propose `--rtt-factor` pour `run` et `ping`. Une valeur invalide
est un échec; `resume` et `replay` refusent cette option pour préserver la politique
enregistrée. Le budget de temps de `ping` suit aussi son attente étendue.

```sh
cargo run -p sns-desktop --locked --offline -- --help --language fr
cargo run -p sns-desktop --locked --offline -- --check-config --rtt-factor 5
cargo run -p sns-cli --locked --offline -- run fixtures/valid/response-wait-scenario.json --rtt-factor 5 --until-us 6000000 --trace /tmp/response-wait.jsonl --recording /tmp/response-wait-recording.json
cargo run -p sns-cli --locked --offline -- replay /tmp/response-wait-recording.json --trace /tmp/response-wait-replayed.jsonl
```

## Validation

Les tests `sns-engine/tests/timing.rs`, contrats, runner CLI, lancement natif et
configuration du workbench couvrent le réseau lent, versions incompatibles,
anciennes fixtures, reprise/rejeu, durées conservées et les six langues/deux vues.
La suite GPU courante compare ses références existantes sans mise à jour.
Voir le [rapport de qualification](../verification/J7-ATTENTES.md) pour les résultats.

F-CFG-005 reste en cours pour v/l/n/s et le journal annoncé. La tranche ne
qualifie ni livraison trois OS ni sélecteur natif; aucune dépendance tierce ajoutée.
