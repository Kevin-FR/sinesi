# Missions évaluables

Exigence F-ADV-004, réalisée dans le parcours élève (F-UX-001 à F-UX-006).
Contrats : [mission](../../contracts/mission.schema.json) et
[correction](../../contracts/mission-report.schema.json).

## Principe

Une mission part d’un réseau volontairement en panne. L’élève lit la consigne, répare
le réseau avec les outils ordinaires de l’atelier, puis le teste. Les objectifs sont
vérifiés automatiquement; un objectif non atteint est expliqué par le texte de
l’auteur. La vérification lit seulement le projet appliqué et la trace de simulation :
elle ne lance, n’avance ni ne modifie aucune simulation.

## Format (version 1)

- `id` en minuscules, chiffres et tirets; `chapter` et `order` placent la mission.
- `title`, `brief`, textes des objectifs, explications et indices : une traduction par
  langue (fr, en, es, it, pt, de), le français étant obligatoire et servant de repli.
- `project` : un projet complet, validé comme tout projet ouvert.
- 1 à 12 objectifs, 0 à 5 indices. Un objectif nomme des équipements existants.

## Contrôles

| Type | Source | Atteint quand |
| --- | --- | --- |
| `gateway` | configuration | la passerelle de l’équipement vaut la valeur attendue |
| `dns_server` | configuration | le serveur DNS de l’équipement vaut la valeur attendue |
| `connected` | topologie | un câble relie les deux équipements |
| `echo_reply` | trace | l’équipement a reçu une réponse d’écho de l’adresse |
| `dns_resolved` | trace | l’équipement a reçu une réponse DNS réussie, non vide, pour le nom |
| `service_listening` | trace | l’application a ouvert son point d’écoute sur l’équipement |

Les contrôles de configuration sont réévalués en permanence : défaire une réparation
rend l’objectif de nouveau incomplet. Un contrôle de trace observé pendant la mission
reste acquis, car revenir à Construire efface la trace de la simulation suivante.

## Étoiles et progression

Une mission réussie vaut trois étoiles, moins une par indice affiché, au minimum une.
La meilleure note est conservée dans le profil utilisateur, jamais dans le projet.
La mission suivante est débloquée dès que la précédente a au moins une étoile.
Changer de document termine la mission en cours; les modifications non enregistrées
sont protégées par la confirmation habituelle.

## Correction exportable

La correction indique la mission, la langue, l’état complet, les indices comptés,
les étoiles (absentes tant que la mission n’est pas réussie) et, pour chaque
objectif, son texte, son état et l’explication s’il n’est pas atteint. La ligne de
commande produit la même correction à partir d’une mission, d’un projet et d’une
trace enregistrée; son code de sortie vaut 0 si la mission est réussie, 2 sinon.

## Missions de l’enseignant

Une mission au même format s’ouvre depuis la carte des missions, aux deux niveaux.
Le fichier est lu et validé sans bloquer l’interface; un fichier invalide est refusé
sans toucher au travail en cours, et un résultat arrivé après une modification du
projet est ignoré. La mission apparaît dans la section « De la classe », toujours
débloquée; rouvrir une mission de même identifiant la remplace.

## Missions intégrées

| Chapitre | Mission | Panne | Objectifs |
| --- | --- | --- | --- |
| 1 | Le câble oublié | un poste sans câble | câble posé, ping réussi |
| 1 | La passerelle oubliée | hôte sans passerelle, deux routeurs | passerelle, ping routé |
| 2 | L’annuaire en panne | service DNS arrêté, mauvais serveur DNS | service démarré, serveur DNS, nom résolu |

## Recettes

| Recette | Attendu |
| --- | --- |
| R-MIS-001 | Les missions intégrées et leurs corrections respectent leurs contrats; un type de contrôle inconnu, une version inconnue, un équipement absent, un identifiant dupliqué ou un titre sans français sont refusés. |
| R-MIS-002 | Panne DNS : sans réparation, les trois objectifs sont incomplets et expliqués; serveur démarré seul, seul le premier est atteint; réparé, la mission est réussie avec trois étoiles, sur une simulation réelle. |
| R-MIS-003 | Passerelle et câble : réussite sur simulation réelle, étoiles diminuées par les indices et bornées par leur nombre, acquis de trace conservé. |
| R-MIS-004 | Dans l’atelier, en six langues : carte, mission verrouillée, vérification expliquée, indice, réparation par le câble guidé, ping, réussite à deux étoiles enregistrées, explication retirée après réparation, mission suivante débloquée et chargée après confirmation. |
| R-MIS-006 | Une mission d’enseignant valide s’ouvre, installe son projet, démarre et figure une seule fois dans « De la classe » même rouverte; une mission invalide est refusée sans changer le projet; un résultat devenu périmé est ignoré. |
| R-MIS-005 | La ligne de commande produit la correction incomplète (code 2) puis réussie (code 0) après une réparation et une trace de ping réelles; une langue inconnue est refusée. |
