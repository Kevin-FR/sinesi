# Session et récupération — J7

F-PROJ-003/006 distinguent trois stockages : préférences utilisateur, projet explicitement enregistré et récupération par instance. Le [rapport de qualification](../verification/J7-SESSION.md) relie ces règles aux preuves; l’import partiel garde le lot projets J7 ouvert.

## Parcours et séparation

Les préférences actuelles conservent la langue hors du projet. Une instance active réserve son propre document de récupération dans l’espace local de l’application; elle ne remplace ni le projet utilisateur ni la récupération d’une autre instance. L’origine du projet est une chaîne d’affichage inactive : elle ne sert jamais à ouvrir ou écrire un fichier.

Sans fichier de lancement, l’application propose les sessions inactives disponibles. L’utilisateur peut reprendre, supprimer explicitement ou reporter. Un projet fourni au lancement est chargé en priorité et n’est jamais remplacé automatiquement; le bouton de récupération reste disponible. Une erreur de lecture ou de validation conserve le document courant et les octets de la récupération refusée.

Une reprise ouvre un document **non enregistré**, sans destination de sauvegarde et en conception. Elle restitue la langue depuis les préférences, le projet validé, Icônes/Schéma, zoom, déplacement, sélection et les brouillons décrits ci-dessous. L’utilisateur choisit ensuite une destination pour enregistrer son travail. La récupération de document n’est pas un checkpoint réseau : files de paquets, caches actifs, progression des applications et connexions réelles ne sont pas relancés. Les [checkpoints](REPRISE_J7.md) restent le format de reprise de l’exécution complète.

## Publication automatique et fermeture

Après initialisation, l’application vérifie toutes les **deux secondes** de temps réel si le document, la vue, l’origine ou les brouillons ont changé. Les commandes utilisateur en attente et les opérations de remplacement sont acquittées avant la capture. Un instantané immuable est validé puis publié en arrière-plan; un contenu identique n’est pas réécrit. Un document initial intact n’est pas publié pendant la proposition d’anciennes sessions. Le statut « une récupération est disponible » signale une publication réussie, sans prétendre que la dernière frappe est déjà durable.

La fermeture normale propose aussi de quitter en gardant la récupération. Elle fige les éditions, attend les opérations en cours, publie le dernier état puis ferme seulement après confirmation de l’écriture. Elle ne nécessite pas d’enregistrer dans le fichier projet. Une erreur conserve la fenêtre ouverte avec un diagnostic. Abandonner explicitement le travail supprime seulement la récupération appartenant à cette instance. Les réservations vides d’une instance sans travail sont supprimées à sa fermeture.

Un arrêt forcé peut perdre les modifications postérieures au dernier instantané publié. La prochaine instance récupère le dernier instantané durable, avec le fichier projet explicitement enregistré intact. Les recettes arrêtent réellement deux processus distincts : stockage puis interface avec temporisation automatique de production.

## Contrat de récupération 1

Le [schéma autonome](../../contracts/session.schema.json) et sa [fixture](../../fixtures/valid/session.json) définissent l’enveloppe. La validation sémantique et celle des brouillons complètent le schéma avant publication ou adoption.

| Champ | Règle |
| --- | --- |
| `session_version` | Entier 1; une autre version est refusée |
| `session_id` | Identité locale `session-…`, caractères ASCII alphanumériques ou tiret, au plus 100 octets |
| `saved_at_unix_ms`, `revision` | Entiers décimaux encodés en chaînes; date de stockage indépendante du temps virtuel |
| `origin_display` | Chaîne inactive optionnelle, au plus 4 096 octets |
| `project` | Projet portable validé, profils 1 à 9, au plus 8 Mio |
| `view` | `diagram`, `zoom` entre 0,25 et 3, `pan` fini dans ±10 000 000, sélection sans doublon dont chaque identité existe |
| `edit_state` | Objet inactif optionnel, au plus 4 Mio; absent dans les premiers documents de session 1 |

L’enveloppe entière est limitée à **12 Mio + 64 Kio**. Les membres inconnus de l’enveloppe et des structures de brouillon typées sont refusés. Les anciennes sessions sans `edit_state` restent acceptées avec des brouillons neufs.

### Brouillons inactifs version 1

`edit_state` contient `version: 1`, `inspector`, `desktop`, `routing`, `security`, `conditions`, `annotation`, `target` et `ttl`; le brouillon d’annotation optionnel `annotation_draft` conserve les saisies ajoutées par le [profil documentaire](ANNOTATIONS_J7.md). L’inspecteur optionnel conserve la paire équipement/configuration locale saisie, y compris les valeurs non appliquées. L’identité, le type d’équipement et les identités d’interfaces doivent correspondre au projet. Les textes d’annotation et de cible sont limités à 4 096 octets chacun.

`desktop` contient l’hôte ouvert et une liste `drafts` de `{device, application, state}`. Chaque application appartient à l’équipement déclaré; les couples sont uniques, au plus 4 096. Les champs de formulaire web à clés composées sont des listes de paires, sans doublon. Les états conservent notamment saisie de terminal non exécutée, texte/path d’éditeur non enregistré, paramètres applicatifs en cours d’édition, destinataires/sujet/corps d’un courrier et formulaires P2P. Les tableaux temporaires de filtrage/passerelle sont limités à 256 lignes et leurs références d’hôtes sont contrôlées.

Ces données restent des **saisies**, même si une valeur métier est invalide. Restaurer ne clique pas sur Appliquer, n’exécute aucune commande, n’envoie aucun message et ne lance aucun téléchargement. Les fenêtres de conditions sont refermées. Textures, images de rendu, historique d’affichage, actions différées, indicateurs d’acquittement d’envoi et handles actifs sont exclus ou réinitialisés. Un nouvel état réseau ne doit pas effacer un nouveau brouillon de courrier à cause d’un ancien acquittement.

## Atomicité, concurrence et cache borné

La publication prépare un fichier sur le même volume, le synchronise puis le remplace atomiquement. Un verrou compagnon distinct protège la session active pendant le remplacement; sa libération explicite à la fermeture du propriétaire, ou par le système après arrêt forcé, rend le document accessible. La lecture et la suppression acquièrent ce même verrou. Le choix d’une ligne de proposition ne réserve pas indéfiniment le document : si une autre instance le reprend d’abord, l’action échoue proprement.

La création et le nettoyage des réservations sont sérialisés par un verrou de répertoire. Une allocation réserve simultanément les places pour le verrou et le fichier vide; elle refuse un cache qui dépasserait **128 entrées**. L’énumération est bornée à **256 entrées**, avec au plus **32 propositions**, classées de la plus récente à la plus ancienne; une limite atteinte est signalée. Les résumés sont construits en ne gardant qu’un projet décodé à la fois.

Le nettoyage automatique concerne uniquement les réservations vides inactives et leurs verrous orphelins vides, jamais un document non vide corrompu. Les sessions actives sont exclues des propositions. Un cache plein autorise encore la reprise d’un document existant en réutilisant son propre emplacement, sans supprimer les autres documents. Lorsqu’une reprise utilise un nouvel emplacement, l’ancienne copie n’est supprimée qu’après publication durable de la nouvelle.

Les enregistrements de projet et exports ne peuvent pas cibler le stockage géré des sessions ni le fichier de préférences. Les exports refusent aussi le projet actuellement ouvert. Une préférence dont le chemin désigne le projet explicite est désactivée en écriture; une préférence invalide est conservée avec diagnostic et valeurs de repli en mémoire. Un changement de langue ne peut donc pas remplacer le projet.

## Limites et suite

La langue est le seul réglage global actuellement persistant; polices, taille d’icônes et disposition restent F-CFG-003. Les options complètes de lancement restent F-CFG-005. Cette qualification utilise le harnais graphique et des arrêts de processus réels sur macOS; elle ne vaut pas parcours natif ni preuve Windows/Linux. Les préférences complètes et la livraison restent dans les lots plateforme/qualité. La [matrice](../suivi/MATRICE.md) conserve le lien avec l’adaptateur.
