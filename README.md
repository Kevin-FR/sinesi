# Sinesi — Simple network simulator

Sinesi est une application opensource de simulation réseau pédagogique, écrite en Rust, avec une interface native egui/eframe 0.37.

**J1 à J6 sont réalisés.** Édition de topologie, Ethernet/ARP/IPv4/ICMP, TCP/UDP, logiciels et fichiers, terminal, routage statique, transfert/TTL, traceroute et RIP pédagogique. Courrier SMTP/POP3 et relais MX, P2P, Wi-Fi par SSID et modem explicite entre instances complètent le socle. L’interface, l’aide et les diagnostics couvrent FR/EN/ES/IT/PT/DE; les vues Icônes/Schéma sont conservées.

Les assets fournis ajoutent les vues **Icônes/Schéma** et un exemple **Salle de classe**. Les symboles ne déclenchent pas de simulation d’impression ou de Wi-Fi. Voir le [rapport J3](docs/verification/J3.md), son [profil portable](docs/agnostique/ROUTAGE_J3.md) et l’[état de reprise](docs/suivi/ETAT.md). J4 ajoute DHCP, DNS et HTTP sur le réseau simulé, avec un navigateur pédagogique, des images et formulaires. Voir le [profil de services](docs/agnostique/SERVICES_J4.md) et les [preuves J4](docs/verification/J4.md). Les logiciels de courrier et P2P restent à J6.

J7 est en cours : reprise/rejeu, récupération, annotations et exports PNG/PDF sont
qualifiés localement. Le [rapport PDF](docs/verification/J7-RAPPORT.md) couvre le
schéma, les configurations et les paquets enregistrés par MAC, en six langues et
dans les deux vues. Import complet, configuration et livraison restent ouverts.

## Lancer le socle

Le toolchain Rust 1.95.0 est défini dans `rust-toolchain.toml` et les dépendances dans `Cargo.lock`.

```sh
# Atelier graphique : édition et simulation du LAN
cargo run -p sns-desktop --locked

# Ping sur le même moteur, sans dépendance graphique
cargo run -p sns-cli --locked -- ping fixtures/valid/lan.json --source host-a --target 192.168.1.3 --count 2 --trace target/lan.jsonl

# Qualification historique J0
cargo run -p sns-cli --locked -- run fixtures/valid/scenario.json --seed 42 --trace /tmp/sinesi-trace.jsonl
cargo run -p sns-cli --locked -- run fixtures/valid/scenario.json --until-us 999
cargo run -p sns-cli --locked -- validate fixtures/valid/project.json
```

Après un premier téléchargement des dépendances, ces commandes acceptent aussi `--offline` placé avant `--`. Le CLI retourne 0 pour une exécution terminée ou arrêtée à la borne virtuelle demandée, 1 pour une erreur et 2 quand le budget d’événements est épuisé. Le JSON de résultat et le fichier de trace permettent de distinguer ces situations.

Dans le menu **Exemples routés**, choisir **Trois réseaux et routes statiques**, **Trois routeurs et RIP** ou **Salle de classe**. Le sélecteur **Icônes/Schéma** change la représentation du même réseau. Dans l’exemple statique : **Simulation → Bureau A → Terminal**, puis `traceroute 192.168.30.2` et **Reprendre**. Les réglages de routes, transfert et RIP sont dans l’inspecteur; la suspension des annonces RIP permet d’observer leur expiration.

Dans l’atelier local, sélectionner **Poste A**, passer en **Simulation**, puis **Envoyer un ping** et **Reprendre**, ou avancer avec **Un événement**. **Échanges LAN** montre les en-têtes et les causes; **Tables ARP / SAT** montre les caches. Les dialogues natifs ouvrent et enregistrent les projets `.sinesi.json`. L’aide F1 décrit le clavier et le câblage.

Pour les applications, charger **Exemple logiciels**, passer en **Simulation**, puis ouvrir **Bureau**. Démarrer le serveur echo sur Portable B et connecter le client de Poste A au port 7777. **Reprendre** exécute les transmissions; **Un événement** permet de les inspecter. Les fichiers modifiés et les installations se sauvegardent dans le projet.

Le navigateur de qualification J0 reste accessible avec `cargo run -p sns-desktop --locked -- --prototype-j0`. Il charge page, image et formulaire via les messages `prototype.message.*`, sans pile TCP/HTTP filaire. Il n’ouvre aucun serveur ou navigateur réel.

## Vérifier et contribuer

Les commandes de contrôle et les tests par domaine sont décrits dans le
[guide de développement](docs/rust/DEVELOPPEMENT.md). Le point d’entrée commun au
local et à la CI est `cargo xtask ci`; les comparaisons graphiques sont distinctes.


Voir [CONTRIBUTING.md](CONTRIBUTING.md), les [contrats implémentés](contracts/README.md) et les rapports [J0](docs/verification/J0.md) / [J1](docs/verification/J1.md). La CI est configurée pour Windows/Linux/macOS; les résultats réellement obtenus sont recensés dans le rapport.

## Cahier des charges et suivi

Commencer par le [guide du dossier](docs/README.md), puis lire le [cahier des charges](docs/agnostique/CAHIER_DES_CHARGES.md) et la [roadmap](docs/agnostique/ROADMAP.md).

| Référence | Contenu |
| --- | --- |
| [Catalogue fonctionnel](docs/agnostique/CATALOGUE.md) | 178 exigences, critères de recette et preuves de code |
| [Architecture agnostique](docs/agnostique/ARCHITECTURE.md) | Découpage du produit et principes de simulation |
| [Matrice de fidélité](docs/agnostique/FIDELITE.md) | Capacités démontrées, simplifications et protocoles futurs |
| [Contrats portables](docs/agnostique/CONTRATS.md) | Modèle, commandes, événements, fichiers et erreurs |
| [Volet Rust](docs/rust/ARCHITECTURE.md) | Architecture native egui/eframe, cœur indépendant et compromis |
| [Pile et dépendances Rust](docs/rust/DEPENDANCES.md) | Bibliothèques, briques Craft, forks et étapes de validation |
| [Plan Rust](docs/rust/PLAN.md) | 50 lots reliés aux exigences |
| [État et reprise](docs/suivi/ETAT.md) | Situation réelle et prochaine tranche de travail |
| [Matrice de traçabilité](docs/suivi/MATRICE.md) | Exigence ↔ jalon ↔ tâche Rust |
| [Audit de référence](docs/audit/METHODE.md) | Périmètre, méthode, limites et divergences observées |

Les fichiers `docs/agnostique/` constituent la spécification réutilisable dans un autre langage. Les détails Rust sont confinés à `docs/rust/`. Les identifiants `F-*` relient les deux; les registres JSON de `docs/suivi/` permettent un suivi exploitable par un agent.


## Licence et référence historique

Le code neuf de Sinesi est sous [licence MIT](LICENSE). L’inventaire est fondé sur l’archive locale Filius 2.14.0; ce corpus, ses actifs et son runtime ne sont pas embarqués dans le dépôt ou le binaire Sinesi. Les références et empreintes restent dans `docs/audit/`. Voir la [décision de provenance](docs/rust/decisions/D-007-provenance.md).

Sans l’archive locale, le contrôle documentaire vérifie la structure du suivi et annonce qu’il ne revalide pas le contenu historique. Pour un contrôle complet : `python3 scripts/docs.py check --require-reference`.

Les 14 SVG sélectionnés du kit graphique fourni sont embarqués dans `assets/network/`; leur [provenance et empreinte](assets/network/provenance.json) sont conservées séparément de la référence Filius.

J5 ajoute le routeur domestique LAN/WAN, le NAT TCP/UDP/ICMP, les redirections, le pare-feu et son administration HTTP simulée. Voir le [profil portable](docs/agnostique/RESEAUX_PRIVES_J5.md) et les [preuves J5](docs/verification/J5.md).

J6 ajoute les applications et connexions avancées : [profil portable](docs/agnostique/APPLICATIONS_J6.md), [adaptateur Rust](docs/rust/J6.md), [preuves J6](docs/verification/J6.md). Les exemples courrier, P2P et Wi-Fi sont accessibles dans **Exemples réseau**. Le modem s’active explicitement en simulation après configuration de son rôle et de son endpoint.
