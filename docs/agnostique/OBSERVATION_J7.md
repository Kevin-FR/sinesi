# Recherche, exports et parcours des messages — J7

Ce profil réalise F-OBS-006 et F-OBS-008. Les vues d’observation sont des projections de faits enregistrés. Rechercher, sélectionner, exporter ou animer un message ne modifie ni le réseau, ni les applications, ni l’aléa, ni le temps virtuel.

## Sélection commune

Les critères se combinent par **ET** : équipement émetteur du fait, borne initiale et finale en microsecondes inclusives, protocole, adresse IP/MAC et corrélation exacte. Un champ vide ne restreint pas la sélection. Le protocole ignore la casse et tient compte des enveloppes enregistrées : un événement Ethernet peut contenir IPv4 et TCP. Les noms correspondent aux familles et aux étiquettes présentes dans la trace; aucun protocole absent n’est déduit du contenu applicatif.

Les adresses correspondent exactement aux champs d’adressage enregistrés, y compris ceux des enveloppes et des extrémités imbriquées. La comparaison des MAC ignore la casse. Il n’existe pas de recherche implicite de préfixe, sous-réseau ou port dans ce champ. La corrélation et l’identité d’équipement sont exactes. La recherche textuelle existante, l’interface, les quatre filtres de couche et le masquage des faits antérieurs restent cumulables avec ces critères. Un temps invalide ou un intervalle inversé affiche un diagnostic, ne sélectionne aucun fait et interdit l’export structuré.

La table et les exports utilisent la même sélection, dans l’ordre de la trace. Une sélection masquée par un filtre n’est plus présentée comme détail courant. Effacer l’affichage conserve la trace canonique; réinitialiser les filtres conserve cette borne d’effacement.

## Exports structurés

JSONL contient un événement complet par ligne. CSV utilise UTF-8, des fins de ligne CRLF et l’échappement des guillemets par doublement. Son en-tête stable est :

```text
run_id,event_id,event_schema_version,sequence,time_us,type,device_id,caused_by,correlation_id,payload_json
```

Le dernier champ porte le contenu structuré complet du fait. Une cause absente est un champ vide. Les identifiants, versions, séquences et temps restent ceux de l’original : aucune renumérotation ni traduction des données exportées. Le couple exécution/événement identifie le fait; une trace reprise peut conserver les faits de la génération antérieure.

L’export fige la sélection et le contenu au déclenchement. Une modification ultérieure des filtres ou l’arrivée d’autres faits ne change pas ce fichier. L’écriture est atomique et s’effectue en arrière-plan. Un résultat dépassant **64 Mio** est refusé avant publication, et une destination identique au projet ouvert est refusée. Une sélection vide donne un JSONL vide ou un CSV limité à son en-tête.

## Parcours causal local et global

Le parcours part du message sélectionné. Il comprend ses ancêtres et ses descendants reliés par les causes enregistrées; il n’inclut pas les autres branches issues de ses ancêtres au seul motif qu’elles partagent une corrélation. L’ordre suit les séquences. Les références intergénérations d’une reprise sont suivies lorsqu’elles se résolvent sans ambiguïté; une cause manquante ou ambiguë marque le parcours comme partiel. Les identités dupliquées dans une même exécution, cycles, causes futures et temps inversés sont refusés.

La vue locale retient les étapes d’un équipement choisi. La vue globale montre les étapes de tous les équipements du parcours. Les quatre couches restent visibles : application, transport, réseau et liaison. La couche active correspond au fait enregistré; les enveloppes montrent les données effectivement présentes. Une couche absente reste explicitement vide. ICMP relève du réseau; un switch ne reçoit aucune étape applicative inventée.

Lecture, pause, début, précédent et suivant commandent une animation de présentation à **600 ms par étape**, indépendante du temps simulé. La chronologie conserve les visites répétées et les réponses. Une branche de diffusion reste une branche causale; deux récepteurs successifs dans la chronologie ne deviennent pas une connexion physique. La vue du réseau souligne l’équipement de l’étape courante dans les vues Icônes et Schéma. Le parcours est figé à son ouverture; un changement d’exécution le ferme.

Les six langues FR EN ES IT PT DE exposent les mêmes contrôles. Les types et identités techniques de la trace restent stables. La [matrice](../suivi/MATRICE.md) relie ces comportements à l’adaptateur; les preuves sont dans le [rapport de qualification](../verification/J7-OBSERVATION.md).

## Recettes

Sur un échange DNS/HTTP simulé, combiner tous les critères et comparer exactement les identifiants et contenus de la table, du JSONL et du CSV. Modifier les filtres pendant l’écriture pour vérifier la sélection figée. Vérifier les caractères accentués, guillemets, retours à la ligne, sélection vide et quota de taille.

Sélectionner ensuite un message, parcourir les étapes locales, puis animer les étapes globales et les quatre couches. Comparer l’état complet du moteur avant/après. Refaire le parcours après reprise, avec des faits conservés d’une ancienne génération; vérifier séparément une cause absente, une autre branche de même corrélation et des références invalides.
