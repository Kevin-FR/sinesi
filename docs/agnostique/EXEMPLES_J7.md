# Exemples pédagogiques J7

Ouvrir un projet depuis **Exemples réseau** ou depuis l’onglet **Exemples** de
l’aide. Les accès directs **Exemple LAN** et **Exemple logiciels** restent
dans la barre principale. Pour les recettes, passer en simulation et avancer
jusqu’aux échanges indiqués. Les noms de postes et adresses sont fournis par
chaque projet. Les consignes de l’aide sont disponibles en FR EN ES IT PT DE.

| Famille | Projet prêt à ouvrir | Action et résultat à observer |
| --- | --- | --- |
| Connexion directe | [direct.json](../../fixtures/valid/direct.json) | Envoyer un ping du poste A vers 192.168.1.3. Sur le câble unique, observer la demande ARP, sa réponse et le retour ICMP. |
| LAN | [lan.json](../../fixtures/valid/lan.json) | Envoyer le même ping depuis A via le commutateur. Consulter la table MAC puis répéter pour distinguer la résolution initiale et le cache. |
| Routes statiques | [routed.json](../../fixtures/valid/routed.json) | Dans le terminal de A, exécuter `traceroute 192.168.30.2`. Identifier les deux routeurs, le TTL et la route de chaque saut. |
| RIP | [rip.json](../../fixtures/valid/rip.json) | Laisser circuler les annonces puis envoyer un ping de A vers 192.168.30.2. Suspendre les annonces d’un routeur et observer l’expiration des routes apprises. |
| DHCP | [services.json](../../fixtures/valid/services.json) | Avancer jusqu’aux baux des postes A et C. Vérifier l’adresse réservée 192.168.1.10 de A et le DNS annoncé 192.168.1.3. |
| DNS | [services.json](../../fixtures/valid/services.json) | Démarrer le serveur DNS. Depuis le terminal du poste A, exécuter `host www.ecole.test` deux fois. Observer la réponse puis le cache. |
| Web | [services.json](../../fixtures/valid/services.json) | Démarrer DNS et le serveur web. Dans le navigateur du poste A, ouvrir `http://www.ecole.test/`. Vérifier la page École, son image et les échanges HTTP. |
| Courrier | [mail.json](../../fixtures/valid/mail.json) | Démarrer les serveurs de courrier Alpha et Beta et le DNS de Serveur Alpha. Depuis le client du poste A, envoyer un message à `alice@alpha.test`, puis relever sa boîte. Retrouver SMTP et POP3 dans le trafic. |
| NAT | [private.json](../../fixtures/valid/private.json) | Démarrer le serveur Echo WAN sur le port 7777. Installer le client TCP sur A et B puis ouvrir deux connexions vers 198.51.100.10:7777. Comparer les adresses et ports avant et après traduction. |
| Pare-feu | [private.json](../../fixtures/valid/private.json) | Sur le routeur domestique, ajouter une règle de refus TCP/7777 pour 192.168.10.0/24 avant une règle d’autorisation identique. Répéter la connexion et inverser les règles. Observer la première règle appliquée. |
| ARP | [arp.json](../../fixtures/valid/arp.json) | Envoyer un ping de A vers 192.168.1.3. Dans le terminal de C, saisir `arpsend 192.168.1.3 192.168.1.2`. Relancer le ping et constater dans la trace que la trame arrive sur C. |
| P2P | [p2p.json](../../fixtures/valid/p2p.json) | Démarrer A puis joindre A depuis B et B depuis C sur le port 6346. Relier aussi A à C. Depuis A, rechercher `reseau` et télécharger le résultat de C. Comparer le fichier reçu à l’original. |
| Modem | [écoute](../../fixtures/valid/modem-listen.json) et [connexion](../../fixtures/valid/modem-connect.json) | Ouvrir les deux projets dans deux instances du même ordinateur. En simulation, activer d’abord le modem en écoute puis celui en connexion. Une fois connecté, envoyer depuis A un ping vers 192.168.1.3. Déconnecter les modems à la fin. |

Pour le modem, les deux projets proposent `127.0.0.1:50555`. En cas de port
occupé, choisir le même port disponible des deux côtés. L’activation explicite
démarre le pont sur la boucle locale. Le test automatisé choisit un port
disponible et transmet sa valeur au processus client.

Les exemples logiciels, classe et réseau sans fil sont aussi intégrés au
catalogue. Les actions et résultats détaillés de leurs parcours se trouvent
dans l’aide intégrée et les recettes des jalons correspondants.
