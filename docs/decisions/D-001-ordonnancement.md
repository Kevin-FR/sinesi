# D-001 — Ordonnancement déterministe J0

État : implémenté et testé dans le moteur de fondation; aucun protocole réseau complet n’est couvert par cette décision.

L’unité est la microseconde virtuelle, sur entier non signé de 64 bits, sérialisé en chaîne décimale canonique. La file applique l’ordre croissant `(temps, priorité, séquence)`. Une arrivée a la priorité 10, un envoi applicatif la priorité 20. La séquence d’insertion départage les événements simultanés de même priorité; elle ne dépend jamais de l’itération d’une table non ordonnée.

Le générateur est SplitMix64 avec arithmétique modulo 2^64. La graine initialise son état. Pour chaque lien effectivement parcouru ayant un jitter non nul, un tirage fixe le supplément de délai dans `[0, jitter_us]` par réduction modulo. Un lien sans jitter ne consomme pas de tirage. Le chemin de prototype est un parcours en largeur des voisins triés par identifiants de liaison/nœud. Ce choix sert à tester le moteur et ne représente pas le routage IPv4 cible.

Un événement est préparé avant mutation. Erreur de temps, débordement de séquence ou quota laisse inchangés file, temps, trace et état aléatoire. Les files sont bornées à 65 536 événements, les traces à 100 000 faits et le registre de commandes acceptées à 4 096 commandes par exécution. La saturation est une erreur explicite. Il n’y a pas d’éviction silencieuse ni de perte réseau inventée.

Le moteur commence en pause. Un pas consomme au plus un événement. Une exécution reçoit un budget et une borne virtuelle inclusive; la borne ne fait pas avancer artificiellement le temps quand aucun événement n’est exécuté. L’arrêt vide la file. Le redémarrage crée une nouvelle génération et rejette les commandes portant l’ancienne identité.

La déduplication porte sur les commandes acceptées : même identité et même contenu retourne l’acceptation initiale sans rejouer l’effet; une identité réutilisée avec un autre contenu est refusée. J0 n’édite pas le projet à travers cette API : sa révision attendue est 0. Les erreurs ne sont pas mises en cache. Un registre plein refuse les nouvelles commandes jusqu’au redémarrage.

Les traces canoniques normalisent l’identifiant de génération, trient les objets récursivement et conservent l’ordre des faits. Le CLI exporte cette trace JSONL et son SHA-256. Le temps réel, les frames, les adresses mémoire et la présentation n’interviennent pas dans son calcul.

Preuves : tests `determinism.rs` et `runner.rs`, notamment ordre des priorités/FIFO, réorganisation du projet, erreurs transactionnelles, changements de graine, arrêt/générations et exécutions dans deux processus. Les checkpoints complets sont réservés à J7.
