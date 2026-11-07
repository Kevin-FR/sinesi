# Catalogue des exigences

Vue générée depuis [features.json](../suivi/features.json); modifier le registre puis lancer `python3 scripts/docs.py render`. Les critères décrivent le produit cible, même lorsqu’ils corrigent une limite historique. Les états de réalisation et leurs preuves figurent dans le registre.

177 exigences : 148 filius, 23 modernisation, 6 extension_optionnelle.

Une preuve de code établit une présence statique; elle ne prouve ni conformité complète à un standard ni fonctionnement constaté à l’exécution.

## Domaines

| Famille | Domaine | Nombre |
| --- | --- | ---: |
| SIM | [Simulation et moteur](#sim) | 8 |
| TOP | [Édition de topologie](#top) | 10 |
| EQU | [Équipements et interfaces](#equ) | 8 |
| L2 | [Ethernet et ARP](#l2) | 6 |
| IP4 | [IPv4 et ICMP](#ip4) | 5 |
| ROUT | [Routage](#rout) | 6 |
| TCP | [Transport TCP simulé](#tcp) | 5 |
| UDP | [Transport UDP simulé](#udp) | 2 |
| OS | [Bureau et logiciels simulés](#os) | 6 |
| FS | [Fichiers et outils locaux](#fs) | 5 |
| CLI | [Terminal simulé](#cli) | 21 |
| DHCP | [Configuration automatique](#dhcp) | 6 |
| DNS | [Résolution de noms](#dns) | 6 |
| WEB | [Web simulé](#web) | 7 |
| NAT | [Traduction d’adresses](#nat) | 6 |
| FW | [Pare-feu pédagogique](#fw) | 7 |
| MAIL | [Messagerie simulée](#mail) | 8 |
| P2P | [Échange pair à pair](#p2p) | 5 |
| OBS | [Observation et pédagogie](#obs) | 10 |
| DOC | [Annotations et rapports](#doc) | 5 |
| EXT | [Logiciels personnalisés et pont réel](#ext) | 6 |
| PROJ | [Projets et compatibilité](#proj) | 6 |
| CFG | [Préférences, aide et distribution](#cfg) | 8 |
| NFR | [Qualité et contraintes transversales](#nfr) | 9 |
| ADV | [Évolutions hors parité](#adv) | 6 |

<a id="sim"></a>

## SIM — Simulation et moteur

<a id="f-sim-001"></a>

### F-SIM-001 — Conception, simulation et documentation

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir trois espaces de travail avec transitions explicites et cohérentes.
- Recette : Construire un réseau, lancer une simulation, annoter puis revenir en conception sans perdre le projet.
- Composant : `engine`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : aucun.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/GUIMainMenu.java](../../_archive/filius-master/src/main/java/filius/gui/GUIMainMenu.java#L444), ligne 444, `public synchronized void selectMode`.

<a id="f-sim-002"></a>

### F-SIM-002 — Démarrage et arrêt propres

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Démarrer tous les équipements et arrêter services, sockets et traitements transitoires à la sortie de simulation.
- Recette : Après dix cycles conception/simulation, aucun paquet ancien ni port occupé ne subsiste.
- Composant : `engine`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : aucun.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/GUIMainMenu.java](../../_archive/filius-master/src/main/java/filius/gui/GUIMainMenu.java#L507), ligne 507, `private void stopSimulation`.

<a id="f-sim-003"></a>

### F-SIM-003 — Vitesse des échanges

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Régler la vitesse visible sans changer le résultat logique du scénario.
- Recette : La même séquence de commandes produit les mêmes livraisons aux vitesses minimale et maximale.
- Composant : `engine`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : aucun.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/GUIMainMenu.java](../../_archive/filius-master/src/main/java/filius/gui/GUIMainMenu.java#L254), ligne 254, `private void updateLatency`.

<a id="f-sim-004"></a>

### F-SIM-004 — Pertes manuelles de paquets

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Permettre de provoquer des pertes et de consulter les unités abandonnées.
- Recette : Maintenir la commande de perte fait échouer un transfert; son relâchement rétablit les livraisons.
- Composant : `engine`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : aucun.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/SimulationControl.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/SimulationControl.java#L60), ligne 60, `JButton dropButton`.

<a id="f-sim-005"></a>

### F-SIM-005 — Horloge virtuelle déterministe

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Ordonnancer les événements par temps virtuel, priorité et séquence stable; injecter la graine aléatoire.
- Recette : Deux exécutions isolées avec mêmes versions, graine et entrées donnent des traces canoniques identiques.
- Composant : `engine`; jalon : `J0`; état : `verified`.
- Prérequis fonctionnels : aucun.

<a id="f-sim-006"></a>

### F-SIM-006 — Pause et pas à pas

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Suspendre la simulation et avancer exactement un événement ou jusqu’à une borne virtuelle.
- Recette : En pause, aucune minuterie de protocole ne progresse; un pas consomme exactement un événement.
- Composant : `engine`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-sim-007"></a>

### F-SIM-007 — Rejeu et points de reprise

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Enregistrer commandes, entrées externes et état complet nécessaire à une reprise.
- Recette : Une reprise depuis un checkpoint retrouve la même trace suffixe, y compris les expirations.
- Composant : `engine`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-sim-008"></a>

### F-SIM-008 — Pannes paramétrées

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Configurer délai et pertes par liaison avec une graine reproductible.
- Recette : Une perte de 100 % bloque une liaison seule; 0 % la rétablit; un rejeu reproduit les pertes.
- Composant : `engine`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : aucun.


<a id="top"></a>

## TOP — Édition de topologie

<a id="f-top-001"></a>

### F-TOP-001 — Palette et glisser-déposer

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Créer les équipements depuis une palette et les disposer sur le canevas.
- Recette : Chaque type disponible peut être ajouté et déplacé; sa position persiste.
- Composant : `topology`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/GUIEvents.java](../../_archive/filius-master/src/main/java/filius/gui/GUIEvents.java#L232), ligne 232, `public void mausPressedDesignMode`.

<a id="f-top-002"></a>

### F-TOP-002 — Sélection multiple et déplacement

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Sélectionner un ou plusieurs nœuds et déplacer le groupe avec ses liaisons.
- Recette : Un rectangle sélectionne plusieurs nœuds; déplacer le groupe conserve les extrémités connectées.
- Composant : `topology`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/GUIEvents.java](../../_archive/filius-master/src/main/java/filius/gui/GUIEvents.java#L147), ligne 147, `public void mausDragged`.

<a id="f-top-003"></a>

### F-TOP-003 — Suppression des équipements et câbles

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Supprimer une sélection, un câble ou toutes les connexions d’un équipement sans référence orpheline.
- Recette : Supprimer un routeur retire ses câbles et préserve les autres nœuds.
- Composant : `topology`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/GUIEvents.java](../../_archive/filius-master/src/main/java/filius/gui/GUIEvents.java#L720), ligne 720, `public void itemLoeschen`.

<a id="f-top-004"></a>

### F-TOP-004 — Câblage interactif et contraintes

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Prévisualiser une liaison et vérifier extrémités, ports libres, types et doublons.
- Recette : Une liaison valide occupe deux ports; une liaison impossible est rejetée avec une explication.
- Composant : `topology`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/GUIEvents.java](../../_archive/filius-master/src/main/java/filius/gui/GUIEvents.java#L469), ligne 469, `public Kabel createConnection`.

<a id="f-top-005"></a>

### F-TOP-005 — Choix et réaffectation des interfaces

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Afficher les connexions d’un routeur et réaffecter leurs interfaces.
- Recette : Échanger deux ports actualise les associations sans modifier les autres liens.
- Composant : `topology`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JConnectionsDialog.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JConnectionsDialog.java#L591), ligne 591, `private void swapConnection`.

<a id="f-top-006"></a>

### F-TOP-006 — Inspecteur et noms affichés

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Éditer les propriétés; choisir nom libre, adresse IP ou MAC comme libellé d’un hôte.
- Recette : Changer le libellé ne change pas l’identité du nœud ni les échanges.
- Composant : `topology`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.
- Preuve : [_archive/filius-master/src/main/java/filius/hardware/knoten/Host.java](../../_archive/filius-master/src/main/java/filius/hardware/knoten/Host.java#L70), ligne 70, `public String holeAnzeigeName`.

<a id="f-top-007"></a>

### F-TOP-007 — Annuler et rétablir

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Journaliser les commandes d’édition réversibles et offrir annuler/rétablir.
- Recette : Supprimer puis annuler un groupe restaure ses paramètres, positions et liaisons.
- Composant : `topology`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.

<a id="f-top-008"></a>

### F-TOP-008 — Navigation moderne

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Offrir zoom, déplacement du canevas, cadrage du réseau et navigation clavier.
- Recette : Après zoom et cadrage, toutes les extrémités restent sélectionnables; le clavier atteint l’inspecteur.
- Composant : `topology`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.

<a id="f-top-009"></a>

### F-TOP-009 — Dupliquer et aligner

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Dupliquer une sélection avec identités uniques et offrir des outils d’alignement.
- Recette : Deux copies possèdent des identifiants et MAC distincts; leurs adresses IP conflictuelles sont signalées.
- Composant : `topology`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.

<a id="f-top-010"></a>

### F-TOP-010 — Raccourcis de travail

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Fournir les raccourcis nouveau/ouvrir/enregistrer, changement de mode, vitesse, aide et outil câble.
- Recette : Les raccourcis documentés déclenchent les mêmes commandes que les boutons et respectent le contexte de saisie.
- Composant : `topology`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-SIM-005.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/JMainFrame.java](../../_archive/filius-master/src/main/java/filius/gui/JMainFrame.java#L139), ligne 139, `case 78: // N (new)`.


<a id="equ"></a>

## EQU — Équipements et interfaces

<a id="f-equ-001"></a>

### F-EQU-001 — Ordinateur et portable

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Fournir deux représentations d’hôtes avec pile réseau et catalogue de logiciels.
- Recette : Un ordinateur et un portable exécutent le même test ping avec des identités distinctes.
- Composant : `devices`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java#L108), ligne 108, `addHardwareComponent(RECHNER`.

<a id="f-equ-002"></a>

### F-EQU-002 — Commutateur Ethernet

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Fournir un commutateur multiport reliant les hôtes du même LAN.
- Recette : Trois hôtes connectés au switch échangent sur le même sous-réseau.
- Composant : `devices`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java#L110), ligne 110, `addHardwareComponent(SWITCH`.

<a id="f-equ-003"></a>

### F-EQU-003 — Routeur multi-interface

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Fournir un routeur avec nombre d’interfaces configurable, minimum deux.
- Recette : Deux sous-réseaux distincts sont routés par des interfaces configurées séparément.
- Composant : `devices`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java#L114), ligne 114, `addHardwareComponent(VERMITTLUNGSRECHNER`.

<a id="f-equ-004"></a>

### F-EQU-004 — Routeur domestique LAN/WAN

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Fournir un équipement distinct à deux interfaces LAN/WAN et services associés.
- Recette : Les ports LAN et WAN restent identifiables au câblage et après sauvegarde.
- Composant : `devices`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java#L112), ligne 112, `addHardwareComponent(GATEWAY`.

<a id="f-equ-005"></a>

### F-EQU-005 — Modem entre instances

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Fournir un pont explicite entre réseaux simulés sur deux instances réelles.
- Recette : Deux instances reliées par leurs modems échangent des trames; déconnecter le pont arrête les échanges.
- Composant : `devices`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/GUIDesignSidebar.java#L115), ligne 115, `addHardwareComponent(MODEM`.

<a id="f-equ-006"></a>

### F-EQU-006 — Paramétrage IPv4 par interface

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer IPv4, masque, passerelle et DNS; afficher et valider les MAC et paramètres effectifs.
- Recette : Un masque invalide est refusé; les paramètres valides sont visibles depuis l’hôte.
- Composant : `devices`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JHostKonfiguration.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JHostKonfiguration.java#L105), ligne 105, `private void aenderungenAnnehmen`.

<a id="f-equ-007"></a>

### F-EQU-007 — Wi-Fi pédagogique par SSID

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Associer un hôte à un switch/point d’accès par SSID, avec choix LAN/Wi-Fi.
- Recette : Changer le SSID du point d’accès retire les associations concernées; choisir le nouveau SSID les recrée.
- Composant : `devices`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JHostKonfiguration.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JHostKonfiguration.java#L564), ligne 564, `private void updateWifiConnection`.

<a id="f-equ-008"></a>

### F-EQU-008 — Apparence nuage du switch

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Permettre une représentation nuage du switch sans changer sa sémantique.
- Recette : Le changement d’icône conserve ports et table MAC.
- Composant : `devices`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JSwitchKonfiguration.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JSwitchKonfiguration.java#L100), ligne 100, `public void changeAppearance`.


<a id="l2"></a>

## L2 — Ethernet et ARP

<a id="f-l2-001"></a>

### F-L2-001 — Trames Ethernet et diffusion

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Encapsuler les paquets avec MAC source/destination et distinguer unicast et broadcast.
- Recette : Un broadcast ARP atteint tous les ports pertinents; un unicast reste adressé à sa destination.
- Composant : `link`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-002, F-EQU-006.
- Preuve : [_archive/filius-master/src/main/java/filius/software/netzzugangsschicht/Ethernet.java](../../_archive/filius-master/src/main/java/filius/software/netzzugangsschicht/Ethernet.java#L46), ligne 46, `public class Ethernet`.

<a id="f-l2-002"></a>

### F-L2-002 — Apprentissage et transfert du switch

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Apprendre les MAC sources; transférer sur le port connu et inonder les destinations inconnues.
- Recette : Après un échange initial, le trafic unicast connu ne ressort pas sur les ports inutiles.
- Composant : `link`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-002, F-EQU-006.
- Preuve : [_archive/filius-master/src/main/java/filius/software/netzzugangsschicht/SwitchPortBeobachter.java](../../_archive/filius-master/src/main/java/filius/software/netzzugangsschicht/SwitchPortBeobachter.java#L42), ligne 42, `public class SwitchPortBeobachter`.

<a id="f-l2-003"></a>

### F-L2-003 — Vieillissement et purge de table MAC

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer la rétention, afficher les mises à jour et vider la SAT.
- Recette : Une entrée expire à la durée choisie; une purge vide immédiatement la table.
- Composant : `link`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-002, F-EQU-006.
- Preuve : [_archive/filius-master/src/main/java/filius/software/system/SwitchFirmware.java](../../_archive/filius-master/src/main/java/filius/software/system/SwitchFirmware.java#L240), ligne 240, `public void checkSAT`.

<a id="f-l2-004"></a>

### F-L2-004 — Prévention pédagogique des boucles

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Borner les circulations et signaler les trames répétées comme dans la protection simplifiée de Filius.
- Recette : Une boucle de switches ne fait pas croître indéfiniment la file; l’abandon est expliqué.
- Composant : `link`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-002, F-EQU-006.
- Preuve : [_archive/filius-master/src/main/java/filius/software/system/SwitchFirmware.java](../../_archive/filius-master/src/main/java/filius/software/system/SwitchFirmware.java#L97), ligne 97, `private LinkedList<EthernetFrame> durchgelaufeneFrames`.

<a id="f-l2-005"></a>

### F-L2-005 — Résolution et cache ARP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Résoudre une IPv4 locale en MAC par requête/réponse et maintenir un cache consultable/purgeable.
- Recette : Une première communication déclenche ARP; la seconde réutilise le cache; sa purge déclenche une nouvelle résolution.
- Composant : `link`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-002, F-EQU-006.
- Preuve : [_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/ARP.java](../../_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/ARP.java#L46), ligne 46, `public class ARP`.

<a id="f-l2-006"></a>

### F-L2-006 — ARP explicite et conflits

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Permettre l’émission pédagogique de réponses ARP avec IP source choisie dans la simulation.
- Recette : Une réponse ARP injectée modifie le cache simulé et apparaît dans la trace sans paquet sur le réseau réel.
- Composant : `link`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-002, F-EQU-006.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L909), ligne 909, `public String arpsend`.


<a id="ip4"></a>

## IP4 — IPv4 et ICMP

<a id="f-ip4-001"></a>

### F-IP4-001 — Adressage et sous-réseaux

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Calculer réseau et broadcast, valider IPv4 et masques contigus, et choisir le prochain saut.
- Recette : 192.168.10.2/24 calcule le réseau .0 et le broadcast .255; un hôte hors réseau passe par la passerelle.
- Composant : `ipv4`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-L2-005.
- Preuve : [_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/IP.java](../../_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/IP.java#L53), ligne 53, `public class IP`.

<a id="f-ip4-002"></a>

### F-IP4-002 — Boucle locale et livraison locale

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Traiter le loopback et la destination propre sans émission externe.
- Recette : Un ping de 127.0.0.1 et de l’adresse propre réussit sans câble.
- Composant : `ipv4`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-L2-005.
- Preuve : [_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/IP.java](../../_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/IP.java#L217), ligne 217, `sendeUnicastLokal`.

<a id="f-ip4-003"></a>

### F-IP4-003 — Transfert IPv4 et TTL

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Décrémenter le TTL à chaque routeur et arrêter une boucle à expiration.
- Recette : Un paquet TTL=1 traversant un routeur produit un abandon et une réponse ICMP explicite.
- Composant : `ipv4`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-L2-005.
- Preuve : [_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/IP.java](../../_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/IP.java#L282), ligne 282, `public void weiterleitenPaket`.

<a id="f-ip4-004"></a>

### F-IP4-004 — ICMP echo et erreurs

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Simuler echo request/reply, time exceeded et destinations inaccessibles du sous-ensemble Filius.
- Recette : Une cible joignable répond; une route manquante et un TTL expiré produisent des résultats distincts.
- Composant : `ipv4`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-L2-005.
- Preuve : [_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/ICMP.java](../../_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/ICMP.java#L47), ligne 47, `public class ICMP`.

<a id="f-ip4-005"></a>

### F-IP4-005 — Broadcast IP et ping de diffusion

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Autoriser les diffusions prévues, avec activation explicite pour ping broadcast.
- Recette : Sans option de diffusion, ping refuse la destination broadcast; avec activation, plusieurs hôtes peuvent répondre.
- Composant : `ipv4`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-L2-005.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L775), ligne 775, `public String ping`.


<a id="rout"></a>

## ROUT — Routage

<a id="f-rout-001"></a>

### F-ROUT-001 — Routes directes et par défaut

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Construire les routes des interfaces locales, du loopback et de la passerelle par défaut.
- Recette : La table distingue les entrées calculées des entrées manuelles.
- Composant : `routing`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-003, F-IP4-003.
- Preuve : [_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/Weiterleitungstabelle.java](../../_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/Weiterleitungstabelle.java#L45), ligne 45, `public class Weiterleitungstabelle`.

<a id="f-rout-002"></a>

### F-ROUT-002 — Routes statiques

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Ajouter, éditer et supprimer destination, masque, passerelle et interface de sortie.
- Recette : Une route plus spécifique l’emporte sur la route par défaut et sa suppression rétablit le chemin précédent.
- Composant : `routing`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-003, F-IP4-003.
- Preuve : [_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/Weiterleitungstabelle.java](../../_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/Weiterleitungstabelle.java#L282), ligne 282, `public Route determineRouteFromStaticRoutingTable`.

<a id="f-rout-003"></a>

### F-ROUT-003 — Activation du transfert sur hôte

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Permettre à un hôte de transférer les paquets quand l’option est activée.
- Recette : Le même paquet est livré avec transfert actif et bloqué lorsque l’option est désactivée.
- Composant : `routing`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-003, F-IP4-003.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JHostKonfiguration.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JHostKonfiguration.java#L120), ligne 120, `bs.setIpForwardingEnabled`.

<a id="f-rout-004"></a>

### F-ROUT-004 — Routage dynamique RIP pédagogique

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Activer le routage automatique avec annonces, métrique en sauts et table de routes.
- Recette : Trois routeurs apprennent une route non directement connectée; le chemin retenu est inspectable.
- Composant : `routing`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-003, F-IP4-003.
- Preuve : [_archive/filius-master/src/main/java/filius/software/rip/RIPBeacon.java](../../_archive/filius-master/src/main/java/filius/software/rip/RIPBeacon.java#L53), ligne 53, `public void announce`.

<a id="f-rout-005"></a>

### F-ROUT-005 — Expiration et routes inaccessibles RIP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Rafraîchir les routes et représenter l’infini à 16 sauts selon le modèle Filius.
- Recette : Une annonce absente finit par rendre sa route inutilisable; le délai est exprimé en temps virtuel.
- Composant : `routing`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-003, F-IP4-003.
- Preuve : [_archive/filius-master/src/main/java/filius/software/rip/RIPTable.java](../../_archive/filius-master/src/main/java/filius/software/rip/RIPTable.java#L42), ligne 42, `public static final int INFINITY`.

<a id="f-rout-006"></a>

### F-ROUT-006 — Consultation locale et web des routes

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Exposer les tables de transfert et de routage dynamique depuis l’UI et l’administration web simulée.
- Recette : Une même route est visible depuis l’inspecteur et la page web du routeur.
- Composant : `routing`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-003, F-IP4-003.
- Preuve : [_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/VermittlungWeb.java](../../_archive/filius-master/src/main/java/filius/software/vermittlungsschicht/VermittlungWeb.java#L60), ligne 60, `createForwardingTablePage`.


<a id="tcp"></a>

## TCP — Transport TCP simulé

<a id="f-tcp-001"></a>

### F-TCP-001 — Sockets clients et serveurs

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Ouvrir, écouter, accepter, connecter et lier une socket à une interface simulée.
- Recette : Un serveur accepte deux clients; chaque connexion est identifiée séparément.
- Composant : `transport`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/software/transportschicht/TCPSocket.java](../../_archive/filius-master/src/main/java/filius/software/transportschicht/TCPSocket.java#L139), ligne 139, `public class TCPSocket`.

<a id="f-tcp-002"></a>

### F-TCP-002 — Établissement TCP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Modéliser SYN, SYN/ACK, ACK et les transitions d’état visibles.
- Recette : Une connexion affiche le handshake dans le bon ordre; une cible muette finit en timeout.
- Composant : `transport`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/software/transportschicht/TCPSocket.java](../../_archive/filius-master/src/main/java/filius/software/transportschicht/TCPSocket.java#L143), ligne 143, `protected static final int CLOSED`.

<a id="f-tcp-003"></a>

### F-TCP-003 — Segmentation, ACK et retransmissions

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Segmenter le contenu, suivre séquence et acquittements et borner les réessais.
- Recette : La perte d’un segment déclenche un réessai; le destinataire reçoit le contenu une seule fois.
- Composant : `transport`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/software/transportschicht/TCPSocket.java](../../_archive/filius-master/src/main/java/filius/software/transportschicht/TCPSocket.java#L443), ligne 443, `senden`.

<a id="f-tcp-004"></a>

### F-TCP-004 — Fermeture et libération

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Modéliser FIN/ACK et fermeture forcée, puis libérer toutes les ressources.
- Recette : Arrêter un serveur permet immédiatement sa réinstallation sur le même port.
- Composant : `transport`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/software/transportschicht/TCPSocket.java](../../_archive/filius-master/src/main/java/filius/software/transportschicht/TCPSocket.java#L655), ligne 655, `public void schliessen`.

<a id="f-tcp-005"></a>

### F-TCP-005 — Allocation et conflits de ports

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Réserver les ports, détecter les collisions et utiliser la plage éphémère 49152–65535.
- Recette : Deux écoutes concurrentes du même endpoint sont refusées; un port libéré est réutilisable.
- Composant : `transport`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/software/transportschicht/TransportProtokoll.java](../../_archive/filius-master/src/main/java/filius/software/transportschicht/TransportProtokoll.java#L44), ligne 44, `public abstract class TransportProtokoll`.


<a id="udp"></a>

## UDP — Transport UDP simulé

<a id="f-udp-001"></a>

### F-UDP-001 — Datagrammes et sockets UDP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Envoyer et recevoir des datagrammes sans connexion fiable.
- Recette : Une perte UDP ne provoque pas de retransmission automatique de transport.
- Composant : `transport`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-IP4-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/transportschicht/UDPSocket.java](../../_archive/filius-master/src/main/java/filius/software/transportschicht/UDPSocket.java#L42), ligne 42, `public class UDPSocket`.

<a id="f-udp-002"></a>

### F-UDP-002 — Diffusion et association de port

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Supporter l’émission broadcast et l’écoute par port/interface.
- Recette : Un datagramme broadcast DHCP atteint les serveurs du LAN et pas un autre LAN sans mécanisme dédié.
- Composant : `transport`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-IP4-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/transportschicht/UDPSocket.java](../../_archive/filius-master/src/main/java/filius/software/transportschicht/UDPSocket.java#L174), ligne 174, `sendeBroadcast`.


<a id="os"></a>

## OS — Bureau et logiciels simulés

<a id="f-os-001"></a>

### F-OS-001 — Bureau par hôte

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Ouvrir un espace propre à chaque hôte avec applications et état réseau.
- Recette : Deux bureaux affichent leurs configurations et fichiers indépendamment.
- Composant : `applications`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001, F-TCP-001, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIDesktopPanel.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIDesktopPanel.java#L64), ligne 64, `public class GUIDesktopPanel`.

<a id="f-os-002"></a>

### F-OS-002 — Installer et désinstaller les 13 logiciels

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Proposer terminal, explorateur, éditeur, images, pare-feu, DNS, serveur/client génériques, serveur/navigateur web, serveur/client mail et P2P.
- Recette : Chaque logiciel apparaît au catalogue; l’installation et la désinstallation modifient la liste de l’hôte.
- Composant : `applications`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001, F-TCP-001, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/resources/config/Desktop_fr_FR.txt](../../_archive/filius-master/src/main/resources/config/Desktop_fr_FR.txt#L1), ligne 1, `Ligne de commande`.

<a id="f-os-003"></a>

### F-OS-003 — Cycle de vie distinct des fenêtres

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Distinguer fenêtre visible, application installée et service actif; rendre les commandes démarrer/arrêter explicites.
- Recette : Fermer la vue d’un serveur ne coupe pas son écoute; arrêter ou désinstaller le serveur la libère.
- Composant : `applications`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001, F-TCP-001, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationWindow.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationWindow.java#L66), ligne 66, `GUIApplicationWindow`.

<a id="f-os-004"></a>

### F-OS-004 — Informations réseau effectives

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Afficher IP, MAC, masque, passerelle, DNS et résultat DHCP actualisés.
- Recette : Après une attribution DHCP, le panneau reflète immédiatement la nouvelle adresse.
- Composant : `applications`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001, F-TCP-001, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/NetworkInfoPanel.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/NetworkInfoPanel.java#L58), ligne 58, `public class NetworkInfoPanel`.

<a id="f-os-005"></a>

### F-OS-005 — Client générique TCP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer cible IP/nom et port, connecter, envoyer du texte et consulter les réponses.
- Recette : Le client reçoit exactement le texte envoyé à un serveur echo.
- Composant : `applications`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001, F-TCP-001, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/clientserver/ClientBaustein.java](../../_archive/filius-master/src/main/java/filius/software/clientserver/ClientBaustein.java#L76), ligne 76, `public class ClientBaustein`.

<a id="f-os-006"></a>

### F-OS-006 — Serveur générique echo

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Démarrer un serveur texte à port configurable avec journal des connexions/messages.
- Recette : Deux clients reçoivent chacun leur message; arrêter le serveur ferme ses sessions.
- Composant : `applications`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001, F-TCP-001, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/clientserver/ServerBausteinMitarbeiter.java](../../_archive/filius-master/src/main/java/filius/software/clientserver/ServerBausteinMitarbeiter.java#L62), ligne 62, `protected void verarbeiteNachricht`.


<a id="fs"></a>

## FS — Fichiers et outils locaux

<a id="f-fs-001"></a>

### F-FS-001 — Système de fichiers par hôte

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Fournir une arborescence isolée avec fichiers texte et binaires.
- Recette : Créer un fichier sur A ne le crée pas sur B; son contenu persiste dans le projet.
- Composant : `filesystem`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/system/Dateisystem.java](../../_archive/filius-master/src/main/java/filius/software/system/Dateisystem.java#L54), ligne 54, `public class Dateisystem`.

<a id="f-fs-002"></a>

### F-FS-002 — Explorateur de fichiers

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Naviguer, créer des dossiers, copier, renommer, supprimer et ouvrir avec l’application adaptée.
- Recette : Un fichier texte copié puis renommé s’ouvre dans l’éditeur avec son contenu intact.
- Composant : `filesystem`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationFileExplorerWindow.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationFileExplorerWindow.java#L75), ligne 75, `public class GUIApplicationFileExplorerWindow`.

<a id="f-fs-003"></a>

### F-FS-003 — Importer un fichier réel

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Importer explicitement un fichier choisi vers le système de fichiers simulé.
- Recette : Un fichier image sélectionné est copié dans le bon hôte sans exposer le reste du disque.
- Composant : `filesystem`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/FileImporter.java](../../_archive/filius-master/src/main/java/filius/software/lokal/FileImporter.java#L80), ligne 80, `public String addFile`.

<a id="f-fs-004"></a>

### F-FS-004 — Éditeur de texte

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Créer, ouvrir, modifier, enregistrer et enregistrer sous un fichier texte virtuel.
- Recette : Modifier index.html change le contenu servi après enregistrement.
- Composant : `filesystem`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationTextEditorWindow.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationTextEditorWindow.java#L140), ligne 140, `speichern`.

<a id="f-fs-005"></a>

### F-FS-005 — Afficheur d’images

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Ouvrir les formats image utiles dans les fichiers virtuels.
- Recette : Une image importée s’affiche sans altérer son contenu binaire.
- Composant : `filesystem`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationImageViewerWindow.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationImageViewerWindow.java#L47), ligne 47, `public class GUIApplicationImageViewerWindow`.


<a id="cli"></a>

## CLI — Terminal simulé

<a id="f-cli-001"></a>

### F-CLI-001 — Aide et erreurs de commande

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Lister les commandes et leur syntaxe, refuser les commandes/arguments inconnus.
- Recette : help ping affiche sa syntaxe et une commande inconnue ne déclenche aucune action.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L631), ligne 631, `public String help(`.

<a id="f-cli-002"></a>

### F-CLI-002 — Navigation des fichiers

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir cd et pwd avec chemins absolus et relatifs.
- Recette : cd vers un dossier puis pwd affiche son chemin; un dossier absent laisse le répertoire courant inchangé.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L486), ligne 486, `public String cd(`.

<a id="f-cli-003"></a>

### F-CLI-003 — Lister les fichiers

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir dir et ls sur le dossier courant.
- Recette : dir et ls donnent les mêmes fichiers et dossiers.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L344), ligne 344, `public String dir(`.

<a id="f-cli-004"></a>

### F-CLI-004 — Créer fichiers et dossiers

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir touch et mkdir dans le système de fichiers virtuel.
- Recette : touch crée un fichier vide et mkdir un répertoire; une collision produit une erreur claire.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L410), ligne 410, `public String touch(`.

<a id="f-cli-005"></a>

### F-CLI-005 — Lire le texte

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir cat et type pour lire un fichier texte.
- Recette : Les deux alias affichent le même contenu et signalent un fichier absent.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L649), ligne 649, `public String cat(`.

<a id="f-cli-006"></a>

### F-CLI-006 — Copier les fichiers

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir copy et cp.
- Recette : La copie conserve le contenu et laisse le fichier source intact.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L255), ligne 255, `public String copy(`.

<a id="f-cli-007"></a>

### F-CLI-007 — Déplacer et renommer

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir move et mv avec validation de la destination.
- Recette : Après un déplacement réussi, seul le chemin destination contient le fichier.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L152), ligne 152, `public String move(`.

<a id="f-cli-008"></a>

### F-CLI-008 — Supprimer fichiers et dossiers

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir del et rm avec refus des opérations invalides.
- Recette : Supprimer une cible existante la retire de l’arborescence sans toucher ses voisins.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L185), ligne 185, `public String del(`.

<a id="f-cli-009"></a>

### F-CLI-009 — Configuration IP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Afficher la configuration IP effective de l’hôte.
- Recette : ipconfig correspond à l’inspecteur, y compris après DHCP.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L279), ligne 279, `public String ipconfig(`.

<a id="f-cli-010"></a>

### F-CLI-010 — Table de routage

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Afficher les routes; ne pas assimiler cette commande à un shell système complet.
- Recette : route montre réseau, masque, passerelle et interface des entrées effectives.
- Composant : `terminal`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L302), ligne 302, `public String route(`.

<a id="f-cli-011"></a>

### F-CLI-011 — Connexions actives

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Afficher endpoints et états TCP ainsi que les sockets UDP.
- Recette : Une écoute affiche LISTEN; une connexion affiche les deux endpoints et son état.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L532), ligne 532, `public String netstat(`.

<a id="f-cli-012"></a>

### F-CLI-012 — Cache ARP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir arp, arp -a IP, arp -d IP et arp -d.
- Recette : La purge ciblée retire une entrée et la purge globale retire toutes les entrées.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L665), ligne 665, `public String arp(`.

<a id="f-cli-013"></a>

### F-CLI-013 — Résolution DNS

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir host et nslookup avec réponses, serveur interrogé et erreurs DNS.
- Recette : host résout A et MX; nslookup expose le résultat A ou un timeout explicite.
- Composant : `terminal`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L699), ligne 699, `public String host(`.

<a id="f-cli-014"></a>

### F-CLI-014 — Ping

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir ping IP/nom, répétitions configurables, statistiques, interruption et diffusion explicite.
- Recette : Un ping normal émet quatre requêtes par défaut; une perte est comptée dans les statistiques.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L775), ligne 775, `public String ping(`.

<a id="f-cli-015"></a>

### F-CLI-015 — Traceroute

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir traceroute par sondes ICMP à TTL croissant.
- Recette : Le premier saut utilise TTL=1 et les routeurs intermédiaires apparaissent dans l’ordre.
- Composant : `terminal`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L935), ligne 935, `public String traceroute(`.

<a id="f-cli-016"></a>

### F-CLI-016 — Capture textuelle

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Afficher en continu les échanges observés sur l’hôte jusqu’à interruption.
- Recette : tcpdump voit un nouvel échange puis s’arrête à la commande d’interruption sans arrêter l’hôte.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L874), ligne 874, `public String tcpdump(`.

<a id="f-cli-017"></a>

### F-CLI-017 — Réponse ARP manuelle

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Exposer arpsend IP-source IP-cible pour les exercices ARP simulés.
- Recette : La trace contient les adresses demandées et aucun socket réel n’est utilisé.
- Composant : `terminal`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L909), ligne 909, `public String arpsend(`.

<a id="f-cli-018"></a>

### F-CLI-018 — Historique et interruption

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Rappeler les commandes au clavier et interrompre une opération longue.
- Recette : Flèche haute rappelle la commande précédente; Ctrl+C arrête ping ou tcpdump sans bloquer la saisie.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationTerminalWindow.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationTerminalWindow.java#L87), ligne 87, `private ArrayList<String> commandHistory`.

<a id="f-cli-019"></a>

### F-CLI-019 — Commande interne test

- Origine : `filius`; état du référentiel : `internal`.
- Exigence : Recenser le helper historique qui écrit test.txt; le réserver au diagnostic, hors interface pédagogique.
- Recette : Le diagnostic est documenté et ne figure pas dans la liste des commandes élèves.
- Composant : `terminal`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java](../../_archive/filius-master/src/main/java/filius/software/lokal/Terminal.java#L616), ligne 616, `public String test(`.

<a id="f-cli-020"></a>

### F-CLI-020 — Fermer le terminal

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir exit pour fermer sa fenêtre sans désinstaller le terminal.
- Recette : exit ferme la vue; rouvrir le terminal conserve les fichiers de l’hôte.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationTerminalWindow.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationTerminalWindow.java#L204), ligne 204, `if (enteredCommand.equals("exit"))`.

<a id="f-cli-021"></a>

### F-CLI-021 — Effacer l’affichage

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir reset et cls pour effacer l’écran et réafficher l’accueil sans réinitialiser le réseau.
- Recette : Après cls, la configuration IP et les services actifs sont inchangés.
- Composant : `terminal`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-FS-001, F-TCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationTerminalWindow.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationTerminalWindow.java#L206), ligne 206, `enteredCommand.equals("reset")`.


<a id="dhcp"></a>

## DHCP — Configuration automatique

<a id="f-dhcp-001"></a>

### F-DHCP-001 — Serveur et plage DHCP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Activer le serveur sur un hôte ou le LAN de la passerelle et configurer sa plage.
- Recette : Deux clients obtiennent deux adresses libres dans la plage; son épuisement est signalé.
- Composant : `dhcp`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dhcp/DHCPServer.java](../../_archive/filius-master/src/main/java/filius/software/dhcp/DHCPServer.java#L53), ligne 53, `public class DHCPServer`.

<a id="f-dhcp-002"></a>

### F-DHCP-002 — Configuration DNS et passerelle distribuée

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Distribuer masque, DNS et passerelle, depuis les réglages du serveur ou une surcharge explicite.
- Recette : Un client applique exactement les paramètres choisis.
- Composant : `dhcp`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dhcp/DHCPServer.java](../../_archive/filius-master/src/main/java/filius/software/dhcp/DHCPServer.java#L70), ligne 70, `private boolean useDhcpSettings`.

<a id="f-dhcp-003"></a>

### F-DHCP-003 — Réservations MAC/IP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer des attributions statiques dans le serveur DHCP.
- Recette : Une MAC réservée reçoit son IPv4 réservée et les autres clients ne la reçoivent pas.
- Composant : `dhcp`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dhcp/DHCPServer.java](../../_archive/filius-master/src/main/java/filius/software/dhcp/DHCPServer.java#L406), ligne 406, `public void addStaticAssignment`.

<a id="f-dhcp-004"></a>

### F-DHCP-004 — Dialogue client et conflits

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Modéliser discover, offer, request, ack/nack, decline et détection ARP des adresses déjà occupées.
- Recette : Un client refuse une offre en conflit et aboutit à une adresse libre ou à une erreur bornée.
- Composant : `dhcp`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dhcp/DHCPClient.java](../../_archive/filius-master/src/main/java/filius/software/dhcp/DHCPClient.java#L51), ligne 51, `public class DHCPClient`.

<a id="f-dhcp-005"></a>

### F-DHCP-005 — Durées d’attribution

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Gérer offres temporaires, attributions expirables et adresses refusées selon le modèle documenté.
- Recette : Une attribution expirée redevient disponible; toutes les durées sont pilotées par l’horloge virtuelle.
- Composant : `dhcp`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dhcp/DHCPServer.java](../../_archive/filius-master/src/main/java/filius/software/dhcp/DHCPServer.java#L58), ligne 58, `private static final long DEFAULT_LEASE_TIME_MILLIS`.

<a id="f-dhcp-006"></a>

### F-DHCP-006 — Repli link-local

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Après échecs DHCP bornés, choisir et vérifier une adresse link-local.
- Recette : Sans serveur, l’hôte obtient une adresse 169.254/16 non conflictuelle; l’échec final reste explicite.
- Composant : `dhcp`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dhcp/DHCPClient.java](../../_archive/filius-master/src/main/java/filius/software/dhcp/DHCPClient.java#L63), ligne 63, `public static final String LINK_LOCAL_ADDRESS_PREFIX`.


<a id="dns"></a>

## DNS — Résolution de noms

<a id="f-dns-001"></a>

### F-DNS-001 — Serveur DNS et enregistrements A

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Démarrer/arrêter un serveur et gérer les associations nom/IPv4.
- Recette : Créer puis supprimer un A change la réponse des nouvelles requêtes après purge/expiration du cache.
- Composant : `dns`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-001, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dns/DNSServer.java](../../_archive/filius-master/src/main/java/filius/software/dns/DNSServer.java#L44), ligne 44, `public class DNSServer`.

<a id="f-dns-002"></a>

### F-DNS-002 — MX et NS

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Éditer et valider les enregistrements mail et serveurs de noms.
- Recette : Un MX désigne un nom et son A permet la livraison inter-domaines; un NS délègue la zone.
- Composant : `dns`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-001, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dns/ResourceRecord.java](../../_archive/filius-master/src/main/java/filius/software/dns/ResourceRecord.java#L61), ligne 61, `public static final String ADDRESS`.

<a id="f-dns-003"></a>

### F-DNS-003 — Résolution hiérarchique et récursion

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Résoudre via serveurs de noms, délégations et choix récursif du modèle Filius.
- Recette : Une requête traverse un NS parent puis le serveur cible et retourne la réponse.
- Composant : `dns`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-001, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dns/Resolver.java](../../_archive/filius-master/src/main/java/filius/software/dns/Resolver.java#L74), ligne 74, `public class Resolver`.

<a id="f-dns-004"></a>

### F-DNS-004 — Enregistrements persistants et champ TTL

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Persister les enregistrements avec leur champ TTL et synchroniser les éditions/suppressions entre vue et stockage.
- Recette : Après sauvegarde/réouverture, le champ TTL est conservé; un enregistrement supprimé ne réapparaît pas lors d’une nouvelle édition.
- Composant : `dns`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-001, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dns/ResourceRecord.java](../../_archive/filius-master/src/main/java/filius/software/dns/ResourceRecord.java#L79), ligne 79, `private int ttl`.

<a id="f-dns-005"></a>

### F-DNS-005 — Noms canoniques et erreurs

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Normaliser casse et point final et exposer absence de réponse, absence de nom et timeout.
- Recette : Example.test et example.test. sont équivalents; un serveur absent produit un timeout borné.
- Composant : `dns`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-UDP-001, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dns/ResourceRecord.java](../../_archive/filius-master/src/main/java/filius/software/dns/ResourceRecord.java#L168), ligne 168, `public void setDomainname`.

<a id="f-dns-006"></a>

### F-DNS-006 — Cache de résolution à expiration TTL

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Ajouter un véritable cache de résolution par nœud avec expiration en temps virtuel et purge explicite.
- Recette : Une réponse est réutilisée avant son TTL puis interrogée de nouveau après expiration; les hôtes ne partagent pas leurs caches.
- Composant : `dns`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-DNS-001, F-SIM-005.


<a id="web"></a>

## WEB — Web simulé

<a id="f-web-001"></a>

### F-WEB-001 — Serveur HTTP et racine web

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Activer un serveur à port configurable et servir les fichiers de son espace virtuel.
- Recette : Un navigateur récupère index.html du serveur et obtient 404 pour une ressource absente.
- Composant : `web`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-004, F-DNS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/www/WebServer.java](../../_archive/filius-master/src/main/java/filius/software/www/WebServer.java#L54), ligne 54, `public class WebServer`.

<a id="f-web-002"></a>

### F-WEB-002 — Messages HTTP GET et POST

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Modéliser requêtes, en-têtes, corps, encodage des chemins et réponses utiles.
- Recette : Un formulaire POST est traité par une page dynamique; une méthode non prise en charge produit 501.
- Composant : `web`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-004, F-DNS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/www/WebServerMitarbeiter.java](../../_archive/filius-master/src/main/java/filius/software/www/WebServerMitarbeiter.java#L123), ligne 123, `protected void verarbeiteNachricht`.

<a id="f-web-003"></a>

### F-WEB-003 — Hôtes virtuels

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Associer plusieurs noms de domaine à des racines distinctes sur un serveur.
- Recette : Deux noms résolus vers la même IP servent deux pages différentes.
- Composant : `web`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-004, F-DNS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/www/WebServer.java](../../_archive/filius-master/src/main/java/filius/software/www/WebServer.java#L221), ligne 221, `public void changeVHostTable`.

<a id="f-web-004"></a>

### F-WEB-004 — Images et types de contenu

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Servir texte/HTML et images PNG, BMP, GIF, JPG avec types adaptés.
- Recette : Une page HTML contenant une image provoque une requête distincte et affiche l’image.
- Composant : `web`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-004, F-DNS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/www/WebServerMitarbeiter.java](../../_archive/filius-master/src/main/java/filius/software/www/WebServerMitarbeiter.java#L100), ligne 100, `HTTPNachricht.IMAGE_PNG`.

<a id="f-web-005"></a>

### F-WEB-005 — Navigateur pédagogique

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir URL, liens et formulaires dans un rendu contrôlé alimenté par la pile simulée.
- Recette : Un clic de lien génère HTTP dans la trace et n’interroge pas le réseau réel.
- Composant : `web`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-004, F-DNS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationWebBrowserWindow.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationWebBrowserWindow.java#L206), ligne 206, `private void abrufenWebseite`.

<a id="f-web-006"></a>

### F-WEB-006 — Ressources et cache isolés

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Isoler les ressources et caches du navigateur par hôte et session.
- Recette : Deux hôtes servant des images homonymes ne partagent pas le cache.
- Composant : `web`; jalon : `J4`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-004, F-DNS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/www/WebBrowser.java](../../_archive/filius-master/src/main/java/filius/software/www/WebBrowser.java#L83), ligne 83, `clearImageCache`.

<a id="f-web-007"></a>

### F-WEB-007 — Pages dynamiques d’administration

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Brancher les pages internes de configuration et de journaux sur le serveur simulé.
- Recette : Une modification depuis le formulaire du routeur change la même configuration que son inspecteur.
- Composant : `web`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-004, F-DNS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/www/WebServerPlugIn.java](../../_archive/filius-master/src/main/java/filius/software/www/WebServerPlugIn.java#L44), ligne 44, `public abstract class WebServerPlugIn`.


<a id="nat"></a>

## NAT — Traduction d’adresses

<a id="f-nat-001"></a>

### F-NAT-001 — Traduction dynamique sortante

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Traduire les connexions LAN vers l’adresse WAN et restaurer les réponses.
- Recette : Deux hôtes LAN ouvrent des connexions vers le même serveur WAN; chaque réponse revient au bon hôte.
- Composant : `nat`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-004, F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/nat/NatGateway.java](../../_archive/filius-master/src/main/java/filius/software/nat/NatGateway.java#L38), ligne 38, `public class NatGateway`.

<a id="f-nat-002"></a>

### F-NAT-002 — Ports externes mappés

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Réallouer les ports externes lorsque plusieurs sessions ont les mêmes ports locaux.
- Recette : Deux sessions en collision obtiennent des mappings distincts et restent fonctionnelles.
- Composant : `nat`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-004, F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/nat/NetworkAddressTranslationTable.java](../../_archive/filius-master/src/main/java/filius/software/nat/NetworkAddressTranslationTable.java#L38), ligne 38, `public class NetworkAddressTranslationTable`.

<a id="f-nat-003"></a>

### F-NAT-003 — Redirection statique de ports

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer port externe, adresse/port interne et protocole TCP/UDP.
- Recette : Un client WAN atteint un service LAN avec un port externe différent du port interne.
- Composant : `nat`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-004, F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JPortForwardingDialog.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JPortForwardingDialog.java#L49), ligne 49, `public class JPortForwardingDialog`.

<a id="f-nat-004"></a>

### F-NAT-004 — ICMP et traceroute à travers NAT

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Traduire les identifiants ICMP, les réponses et les erreurs encapsulées nécessaires.
- Recette : Ping et traceroute depuis deux hôtes LAN traversent NAT sans confondre les réponses.
- Composant : `nat`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-004, F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/nat/NatGatewayWANThread.java](../../_archive/filius-master/src/main/java/filius/software/nat/NatGatewayWANThread.java#L41), ligne 41, `public class NatGatewayWANThread`.

<a id="f-nat-005"></a>

### F-NAT-005 — Consultation, purge et expiration

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Afficher endpoints, ports/identifiants, protocole, nature et durée des entrées NAT.
- Recette : Une session apparaît dans la table; purge et expiration retirent son mapping.
- Composant : `nat`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-004, F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/system/GatewayFirmware.java](../../_archive/filius-master/src/main/java/filius/software/system/GatewayFirmware.java#L353), ligne 353, `public void checkNAT`.

<a id="f-nat-006"></a>

### F-NAT-006 — DHCP sur WAN et serveur côté LAN

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer indépendamment acquisition WAN et distribution LAN.
- Recette : Une passerelle reçoit son WAN par DHCP tout en attribuant une configuration différente sur son LAN.
- Composant : `nat`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-NAT-001, F-DHCP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JGatewayConfiguration.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JGatewayConfiguration.java#L70), ligne 70, `public class JGatewayConfiguration`.


<a id="fw"></a>

## FW — Pare-feu pédagogique

<a id="f-fw-001"></a>

### F-FW-001 — Pare-feu sur hôte et routeur

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Activer/désactiver un filtrage sur les équipements concernés.
- Recette : Le même flux réussit pare-feu désactivé et suit les règles après activation.
- Composant : `firewall`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java](../../_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java#L63), ligne 63, `public class Firewall`.

<a id="f-fw-002"></a>

### F-FW-002 — Règles ordonnées et politique par défaut

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Ajouter, modifier, supprimer et réordonner des règles ACCEPT/DROP avec politique de repli.
- Recette : Deux règles contradictoires suivent la première correspondance; une absence de règle suit la politique.
- Composant : `firewall`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java](../../_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java#L156), ligne 156, `public boolean moveUp`.

<a id="f-fw-003"></a>

### F-FW-003 — Sélection IP, masque, port et protocole

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Filtrer source/destination, réseaux, port destination et TCP/UDP.
- Recette : Une règle ciblée bloque un service sans bloquer un autre port.
- Composant : `firewall`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/firewall/FirewallRule.java](../../_archive/filius-master/src/main/java/filius/software/firewall/FirewallRule.java#L36), ligne 36, `public class FirewallRule`.

<a id="f-fw-004"></a>

### F-FW-004 — Mode SYN et filtrage UDP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Exposer le filtrage TCP limité aux SYN et le traitement des réponses UDP de Filius, sans promettre un suivi de connexion complet.
- Recette : Un test distingue filtrage SYN seul et filtrage de tous les segments; le chemin retour UDP est couvert.
- Composant : `firewall`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java](../../_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java#L72), ligne 72, `private boolean filterSYNSegmentsOnly`.

<a id="f-fw-005"></a>

### F-FW-005 — Politique ICMP

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Autoriser ou bloquer ICMP avec une commande visible.
- Recette : Bloquer ICMP fait échouer ping tout en laissant un HTTP autorisé fonctionner.
- Composant : `firewall`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java](../../_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java#L71), ligne 71, `private boolean dropICMP`.

<a id="f-fw-006"></a>

### F-FW-006 — Journal du filtrage

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Consigner acceptations/refus utiles et numéro de règle appliquée.
- Recette : Un refus est corrélé au paquet et à la règle responsable.
- Composant : `firewall`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-ROUT-002, F-TCP-003, F-UDP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java](../../_archive/filius-master/src/main/java/filius/software/firewall/Firewall.java#L207), ligne 207, `private void notifyRuleApplication`.

<a id="f-fw-007"></a>

### F-FW-007 — Administration web du routeur

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer règles, activation et journal depuis les pages web simulées.
- Recette : Un formulaire change la règle et le journal web reflète son application.
- Composant : `firewall`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-FW-002, F-WEB-007.
- Preuve : [_archive/filius-master/src/main/java/filius/software/firewall/FirewallWebKonfig.java](../../_archive/filius-master/src/main/java/filius/software/firewall/FirewallWebKonfig.java#L41), ligne 41, `public class FirewallWebKonfig`.


<a id="mail"></a>

## MAIL — Messagerie simulée

<a id="f-mail-001"></a>

### F-MAIL-001 — Serveur SMTP et POP3

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer domaine et ports; activer les services et afficher leur activité.
- Recette : Un client envoie via SMTP puis récupère le message via POP3; la trace distingue les sessions.
- Composant : `mail`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-DNS-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/email/EmailServer.java](../../_archive/filius-master/src/main/java/filius/software/email/EmailServer.java#L56), ligne 56, `public class EmailServer`.

<a id="f-mail-002"></a>

### F-MAIL-002 — Comptes et boîtes aux lettres

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Créer, valider, supprimer et persister les comptes et identifiants pédagogiques.
- Recette : Un compte existant se connecte et un compte supprimé est rejeté.
- Composant : `mail`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-DNS-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/email/EmailServer.java](../../_archive/filius-master/src/main/java/filius/software/email/EmailServer.java#L162), ligne 162, `public boolean benutzerHinzufuegen`.

<a id="f-mail-003"></a>

### F-MAIL-003 — Livraison locale et inter-domaines

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Livrer localement ou relayer vers un autre domaine via MX puis A.
- Recette : Un message entre deux domaines traverse le serveur distant et arrive dans la bonne boîte.
- Composant : `mail`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-DNS-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/email/EmailServer.java](../../_archive/filius-master/src/main/java/filius/software/email/EmailServer.java#L466), ligne 466, `public void emailWeiterleiten`.

<a id="f-mail-004"></a>

### F-MAIL-004 — Destinataire inconnu

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Signaler les destinataires non résolus ou inexistants par le comportement d’erreur documenté.
- Recette : Un envoi à un utilisateur absent produit un retour explicite et aucune livraison silencieuse.
- Composant : `mail`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-DNS-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/email/EmailServer.java](../../_archive/filius-master/src/main/java/filius/software/email/EmailServer.java#L497), ligne 497, `public void sendUnknownReceiverResponse`.

<a id="f-mail-005"></a>

### F-MAIL-005 — Client et configuration du compte

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer identité, SMTP/POP3, ports et identifiants sur l’hôte.
- Recette : Deux clients peuvent utiliser des comptes indépendants sans échanger leurs réglages.
- Composant : `mail`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-DNS-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationEmailAnwendungWindow.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/GUIApplicationEmailAnwendungWindow.java#L586), ligne 586, `private void kontoVerwalten`.

<a id="f-mail-006"></a>

### F-MAIL-006 — Rédiger, envoyer et répondre

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Gérer expéditeur, destinataires, copie/CCI, sujet et corps textuel selon le modèle Email.
- Recette : Une réponse reprend le destinataire et le sujet; les champs To/CC/BCC sont conservés sans exposer BCC aux autres destinataires.
- Composant : `mail`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-DNS-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/email/Email.java](../../_archive/filius-master/src/main/java/filius/software/email/Email.java#L41), ligne 41, `public class Email`.

<a id="f-mail-007"></a>

### F-MAIL-007 — Réception, dossiers et suppression

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Récupérer les messages, consulter reçus/envoyés, supprimer et persister leur contenu.
- Recette : Un message envoyé apparaît dans Envoyés et après récupération dans Réception; sa suppression persiste.
- Composant : `mail`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-DNS-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/email/EmailAnwendung.java](../../_archive/filius-master/src/main/java/filius/software/email/EmailAnwendung.java#L211), ligne 211, `public void removeReceivedMail`.

<a id="f-mail-008"></a>

### F-MAIL-008 — Carnet de contacts interne

- Origine : `filius`; état du référentiel : `internal`.
- Exigence : Préserver les capacités de gestion de contacts exposées par le modèle; ne pas supposer une UI complète.
- Recette : Un contact ajouté peut être enregistré, relu et supprimé par l’API applicative.
- Composant : `mail`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-DNS-002, F-OS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/email/EmailAnwendung.java](../../_archive/filius-master/src/main/java/filius/software/email/EmailAnwendung.java#L119), ligne 119, `public boolean kontaktHinzufuegen`.


<a id="p2p"></a>

## P2P — Échange pair à pair

<a id="f-p2p-001"></a>

### F-P2P-001 — Adhésion au réseau

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer le pair d’entrée et le nombre maximal de participants, rejoindre et afficher les voisins connus.
- Recette : Trois pairs connectés apparaissent dans les vues de réseau; une limite de participants configurée est respectée.
- Composant : `p2p`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerAnwendung.java](../../_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerAnwendung.java#L276), ligne 276, `public void beitretenNetzwerk`.

<a id="f-p2p-002"></a>

### F-P2P-002 — Découverte Gnutella simplifiée

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Simuler Ping/Pong, identifiant, TTL et prévention des doublons de messages.
- Recette : Un message en boucle ne circule pas indéfiniment et les pairs découverts sont listés.
- Composant : `p2p`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerPaket.java](../../_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerPaket.java#L38), ligne 38, `public class PeerToPeerPaket`.

<a id="f-p2p-003"></a>

### F-P2P-003 — Partage et recherche de fichiers

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Publier les fichiers du répertoire partagé et propager Query/QueryHit.
- Recette : Une recherche sur un nom partagé par un pair distant retourne une source et son fichier.
- Composant : `p2p`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerAnwendung.java](../../_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerAnwendung.java#L328), ligne 328, `public void sucheDatei`.

<a id="f-p2p-004"></a>

### F-P2P-004 — Téléchargement

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Transférer un résultat vers les fichiers de l’hôte demandeur.
- Recette : Le fichier téléchargé a le même contenu que le fichier partagé.
- Composant : `p2p`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerAnwendung.java](../../_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerAnwendung.java#L304), ligne 304, `public void herunterladenDatei`.

<a id="f-p2p-005"></a>

### F-P2P-005 — Annulation et remise à zéro

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Annuler une recherche, vider les résultats et réinitialiser les connexions du réseau P2P.
- Recette : Annuler arrête la collecte de nouveaux résultats et permet une nouvelle recherche.
- Composant : `p2p`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-TCP-003, F-FS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerAnwendung.java](../../_archive/filius-master/src/main/java/filius/software/dateiaustausch/PeerToPeerAnwendung.java#L343), ligne 343, `public void abbrechenSuche`.


<a id="obs"></a>

## OBS — Observation et pédagogie

<a id="f-obs-001"></a>

### F-OBS-001 — Historique par interface

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Enregistrer envoi/réception avec numéro, temps, interface, direction et protocole.
- Recette : Un échange entre deux hôtes est inspectable depuis chaque interface et garde un identifiant de corrélation.
- Composant : `observation`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-L2-001, F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/rahmenprogramm/nachrichten/Lauscher.java](../../_archive/filius-master/src/main/java/filius/rahmenprogramm/nachrichten/Lauscher.java#L55), ligne 55, `public class Lauscher`.

<a id="f-obs-002"></a>

### F-OBS-002 — Vue agrégée et vue par couches

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Offrir une vue synthétique et une vue détaillée des quatre couches pédagogiques.
- Recette : La même communication est lisible dans les deux vues sans divergence de contenu.
- Composant : `observation`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-L2-001, F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/nachrichtensicht/AggregatedExchangePanel.java](../../_archive/filius-master/src/main/java/filius/gui/nachrichtensicht/AggregatedExchangePanel.java#L64), ligne 64, `public class AggregatedExchangePanel`.

<a id="f-obs-003"></a>

### F-OBS-003 — Inspection des unités de données

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Afficher en-têtes, adresses, ports, flags, TTL, identifiants ICMP et contenu utile.
- Recette : Cliquer sur une trame affiche ses valeurs historiques même après une traduction NAT.
- Composant : `observation`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-L2-001, F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/nachrichtensicht/MessageDetailsTable.java](../../_archive/filius-master/src/main/java/filius/gui/nachrichtensicht/MessageDetailsTable.java#L29), ligne 29, `public class MessageDetailsTable`.

<a id="f-obs-004"></a>

### F-OBS-004 — Filtres de couches, suivi et purge

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Choisir les couches visibles, activer le défilement automatique et vider le journal.
- Recette : Masquer une couche change la vue sans supprimer les événements; purger est une action distincte.
- Composant : `observation`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-L2-001, F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/nachrichtensicht/LayeredMessageTable.java](../../_archive/filius-master/src/main/java/filius/gui/nachrichtensicht/LayeredMessageTable.java#L172), ligne 172, `private void initKontextMenue`.

<a id="f-obs-005"></a>

### F-OBS-005 — Animation d’activité des liens

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Mettre en évidence les liens traversés pendant les échanges.
- Recette : Le chemin d’un ping s’illumine à vitesse lente et les couleurs se réinitialisent à l’arrêt.
- Composant : `observation`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-L2-001, F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JCablePanel.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JCablePanel.java#L219), ligne 219, `resetColor`.

<a id="f-obs-006"></a>

### F-OBS-006 — Parcours entre couches local et global

- Origine : `filius`; état du référentiel : `experimental`.
- Exigence : Visualiser encapsulation/décapsulation et trajet entre équipements.
- Recette : Une sélection de message peut animer son parcours local puis global et montrer les quatre couches.
- Composant : `observation`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-L2-001, F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/schichtensicht/LayerPathDialog.java](../../_archive/filius-master/src/main/java/filius/gui/schichtensicht/LayerPathDialog.java#L18), ligne 18, `public class LayerPathDialog`.

<a id="f-obs-007"></a>

### F-OBS-007 — Inspection SAT et NAT

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Fournir des vues actualisées des tables MAC et NAT.
- Recette : Une nouvelle communication actualise les tables sans rouvrir le panneau.
- Composant : `observation`; jalon : `J5`; état : `not_started`.
- Prérequis fonctionnels : F-L2-003, F-NAT-005.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/anwendungssicht/NatViewer.java](../../_archive/filius-master/src/main/java/filius/gui/anwendungssicht/NatViewer.java#L49), ligne 49, `public class NatViewer`.

<a id="f-obs-008"></a>

### F-OBS-008 — Recherche et export de traces

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Filtrer par équipement, temps, protocole, adresse et corrélation; exporter JSONL et CSV.
- Recette : Une sélection exportée contient exactement les événements filtrés et leurs identifiants stables.
- Composant : `observation`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-L2-001, F-IP4-004.

<a id="f-obs-009"></a>

### F-OBS-009 — Explication des décisions

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Relier chaque transfert/refus à sa route, règle, résolution ou expiration.
- Recette : Cliquer un paquet abandonné affiche une cause structurée et l’équipement responsable.
- Composant : `observation`; jalon : `J3`; état : `not_started`.
- Prérequis fonctionnels : F-L2-001, F-IP4-004.

<a id="f-obs-010"></a>

### F-OBS-010 — Export texte de la vue agrégée

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Exporter le tableau des échanges agrégés en texte UTF-8.
- Recette : Le fichier texte contient les colonnes et les lignes de la vue choisie.
- Composant : `observation`; jalon : `J2`; état : `not_started`.
- Prérequis fonctionnels : F-L2-001, F-IP4-004.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/nachrichtensicht/AggregatedMessageTable.java](../../_archive/filius-master/src/main/java/filius/gui/nachrichtensicht/AggregatedMessageTable.java#L371), ligne 371, `public synchronized void writeToStream`.


<a id="doc"></a>

## DOC — Annotations et rapports

<a id="f-doc-001"></a>

### F-DOC-001 — Zones de texte

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Créer, éditer, déplacer et redimensionner des annotations.
- Recette : Une annotation conserve son texte et ses dimensions après réouverture du projet.
- Composant : `reporting`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001, F-OBS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JDocuElement.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JDocuElement.java#L68), ligne 68, `public class JDocuElement`.

<a id="f-doc-002"></a>

### F-DOC-002 — Rectangles et styles

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Créer des zones structurelles; régler couleurs, taille et style des textes.
- Recette : Une zone colorée reste derrière les équipements; les styles persistent.
- Composant : `reporting`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001, F-OBS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JDocuElement.java](../../_archive/filius-master/src/main/java/filius/gui/netzwerksicht/JDocuElement.java#L68), ligne 68, `public class JDocuElement`.

<a id="f-doc-003"></a>

### F-DOC-003 — Export PNG du réseau

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Exporter une image cadrée du réseau et des annotations avec fond transparent.
- Recette : L’export inclut tous les nœuds et annotations sans les panneaux de l’interface.
- Composant : `reporting`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001, F-OBS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/GUIContainer.java](../../_archive/filius-master/src/main/java/filius/gui/GUIContainer.java#L472), ligne 472, `public void exportAsImage`.

<a id="f-doc-004"></a>

### F-DOC-004 — Rapport PDF

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Exporter schéma, configuration, interfaces, logiciels, DNS, routes et échanges par MAC.
- Recette : Un rapport d’un scénario avec DNS et routeur contient ces sections et les paquets émis/reçus.
- Composant : `reporting`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001, F-OBS-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/documentation/ReportGenerator.java](../../_archive/filius-master/src/main/java/filius/gui/documentation/ReportGenerator.java#L97), ligne 97, `public void generateReport`.

<a id="f-doc-005"></a>

### F-DOC-005 — Export SVG historique retiré

- Origine : `filius`; état du référentiel : `removed`.
- Exigence : Tracer l’ancien export SVG comme fonction retirée de l’interface; une réintroduction est optionnelle.
- Recette : La matrice distingue export SVG retiré et export PNG actif sans annoncer SVG livré.
- Composant : `reporting`; jalon : `J9`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001, F-OBS-001.
- Preuve : [_archive/filius-master/Changelog.md](../../_archive/filius-master/Changelog.md#L149), ligne 149, `Remove svg export`.


<a id="ext"></a>

## EXT — Logiciels personnalisés et pont réel

<a id="f-ext-001"></a>

### F-EXT-001 — Assistant de création de logiciels

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Créer un logiciel client/serveur depuis des modèles et des métadonnées.
- Recette : Un exemple echo personnalisé apparaît dans le catalogue de logiciels.
- Composant : `extensions`; jalon : `J8`; état : `not_started`.
- Prérequis fonctionnels : F-OS-005, F-OS-006.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/quelltextsicht/FrameSoftwareWizard.java](../../_archive/filius-master/src/main/java/filius/gui/quelltextsicht/FrameSoftwareWizard.java#L47), ligne 47, `public class FrameSoftwareWizard`.

<a id="f-ext-002"></a>

### F-EXT-002 — Édition, compilation et diagnostics

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Éditer plusieurs sources, lancer la construction et afficher les erreurs localisées.
- Recette : Une erreur de source est reliée au fichier et à la ligne; une version correcte peut être installée.
- Composant : `extensions`; jalon : `J8`; état : `not_started`.
- Prérequis fonctionnels : F-OS-005, F-OS-006.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/quelltextsicht/PanelCompiler.java](../../_archive/filius-master/src/main/java/filius/gui/quelltextsicht/PanelCompiler.java#L162), ligne 162, `private boolean kompilieren`.

<a id="f-ext-003"></a>

### F-EXT-003 — Gestion et emballage des extensions

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Créer, réouvrir, supprimer et embarquer les extensions avec le scénario sans exécution automatique à l’import.
- Recette : Un scénario exporté référence son extension; à l’ouverture, son activation requiert une action explicite.
- Composant : `extensions`; jalon : `J8`; état : `not_started`.
- Prérequis fonctionnels : F-OS-005, F-OS-006.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/quelltextsicht/PanelVerwaltung.java](../../_archive/filius-master/src/main/java/filius/gui/quelltextsicht/PanelVerwaltung.java#L65), ligne 65, `public class PanelVerwaltung`.

<a id="f-ext-004"></a>

### F-EXT-004 — API des applications et pages web

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Exposer sockets simulées, fichiers virtuels, événements et pages dynamiques via un contrat versionné.
- Recette : Une extension ne peut accéder qu’aux capacités déclarées et fonctionne dans un scénario headless.
- Composant : `extensions`; jalon : `J8`; état : `not_started`.
- Prérequis fonctionnels : F-OS-005, F-OS-006.
- Preuve : [_archive/filius-master/src/main/java/filius/software/www/WebServerPlugIn.java](../../_archive/filius-master/src/main/java/filius/software/www/WebServerPlugIn.java#L44), ligne 44, `public abstract class WebServerPlugIn`.

<a id="f-ext-005"></a>

### F-EXT-005 — Modem client/serveur

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Configurer endpoint réel et port, attendre une connexion, connecter et déconnecter un tunnel de trames.
- Recette : Une paire d’instances transmet un ping simulé; interrompre le tunnel produit un état déconnecté clair.
- Composant : `extensions`; jalon : `J6`; état : `not_started`.
- Prérequis fonctionnels : F-EQU-005, F-L2-001.
- Preuve : [_archive/filius-master/src/main/java/filius/software/system/ModemFirmware.java](../../_archive/filius-master/src/main/java/filius/software/system/ModemFirmware.java#L108), ligne 108, `public synchronized void starteServer`.

<a id="f-ext-006"></a>

### F-EXT-006 — Isolation et quotas des extensions

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Exécuter les extensions dans une frontière isolée avec limites mémoire/instructions et sans accès hôte implicite.
- Recette : Une boucle infinie est interrompue par quota; ni fichiers réels ni réseau réel ne sont accessibles sans capacité dédiée.
- Composant : `extensions`; jalon : `J8`; état : `not_started`.
- Prérequis fonctionnels : F-OS-005, F-OS-006.


<a id="proj"></a>

## PROJ — Projets et compatibilité

<a id="f-proj-001"></a>

### F-PROJ-001 — Nouveau, ouvrir et enregistrer

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Gérer un projet complet avec nom de fichier et protection des modifications.
- Recette : Après enregistrer/rouvrir, topologie, paramètres, logiciels, fichiers et annotations sont équivalents.
- Composant : `projects`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/rahmenprogramm/SzenarioVerwaltung.java](../../_archive/filius-master/src/main/java/filius/rahmenprogramm/SzenarioVerwaltung.java#L108), ligne 108, `public boolean speichern`.

<a id="f-proj-002"></a>

### F-PROJ-002 — Contenu du scénario Filius

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Inventorier le ZIP .fls, sa configuration XML et ses fichiers/extensions embarqués.
- Recette : Les 18 exemples peuvent être analysés en lecture seule et leurs classes inconnues sont signalées.
- Composant : `projects`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/rahmenprogramm/SzenarioVerwaltung.java](../../_archive/filius-master/src/main/java/filius/rahmenprogramm/SzenarioVerwaltung.java#L108), ligne 108, `public boolean speichern`.

<a id="f-proj-003"></a>

### F-PROJ-003 — Préférences et session restaurée

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Séparer préférences utilisateur et projet et proposer une récupération de la dernière session.
- Recette : Un redémarrage restitue les réglages et la dernière session sans écraser un fichier projet explicite.
- Composant : `projects`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.
- Preuve : [_archive/filius-master/src/main/java/filius/Main.java](../../_archive/filius-master/src/main/java/filius/Main.java#L67), ligne 67, `public class Main`.

<a id="f-proj-004"></a>

### F-PROJ-004 — Format de projet portable et versionné

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Définir un schéma neutre avec migrations explicites et sauvegarde atomique.
- Recette : Un projet migré conserve une copie source; un échec d’écriture ne détruit pas la dernière sauvegarde.
- Composant : `projects`; jalon : `J0`; état : `verified`.
- Prérequis fonctionnels : F-SIM-005.

<a id="f-proj-005"></a>

### F-PROJ-005 — Import .fls avec rapport de couverture

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Convertir les données Filius connues sans exécuter XML, bytecode ou sources embarquées.
- Recette : Chaque exemple produit un rapport listant champs convertis, pertes et extensions non exécutées.
- Composant : `projects`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.

<a id="f-proj-006"></a>

### F-PROJ-006 — Autosauvegarde et récupération

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Conserver une récupération atomique distincte du fichier utilisateur.
- Recette : Une fermeture forcée permet de récupérer les dernières modifications en gardant la version enregistrée.
- Composant : `projects`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-TOP-001.


<a id="cfg"></a>

## CFG — Préférences, aide et distribution

<a id="f-cfg-001"></a>

### F-CFG-001 — Français, anglais et allemand

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Localiser l’interface et l’aide et persister la langue préférée.
- Recette : Chaque écran essentiel possède des libellés FR/EN/DE; un projet reste identique après changement de langue.
- Composant : `platform`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-PROJ-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/LanguageSelectionDialog.java](../../_archive/filius-master/src/main/java/filius/gui/LanguageSelectionDialog.java#L95), ligne 95, `private void initializeText`.

<a id="f-cfg-002"></a>

### F-CFG-002 — Aide contextuelle et informations

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Fournir une aide par mode, exemples pédagogiques, version et crédits.
- Recette : L’aide pertinente s’ouvre depuis chaque mode; la version de format et la version du produit sont consultables.
- Composant : `platform`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-PROJ-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/HelpPanel.java](../../_archive/filius-master/src/main/java/filius/gui/HelpPanel.java#L47), ligne 47, `public class HelpPanel`.

<a id="f-cfg-003"></a>

### F-CFG-003 — Icônes et préférences d’affichage

- Origine : `filius`; état du référentiel : `experimental`.
- Exigence : Préserver le choix de jeux d’icônes et améliorer dimensions, police et disposition des fenêtres.
- Recette : Une police agrandie reste lisible et la préférence persiste sans changement de réseau.
- Composant : `platform`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-PROJ-001.
- Preuve : [_archive/filius-master/src/main/java/filius/gui/LanguageSelectionDialog.java](../../_archive/filius-master/src/main/java/filius/gui/LanguageSelectionDialog.java#L83), ligne 83, `private void initializeSettings`.

<a id="f-cfg-004"></a>

### F-CFG-004 — Configuration globale et priorités

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Documenter priorité des valeurs CLI, profil utilisateur et configuration administrateur; gérer les fichiers invalides.
- Recette : Une valeur invalide est diagnostiquée et remplacée par une valeur sûre sans empêcher le lancement.
- Composant : `platform`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-PROJ-001.
- Preuve : [_archive/filius-master/src/main/java/filius/rahmenprogramm/Information.java](../../_archive/filius-master/src/main/java/filius/rahmenprogramm/Information.java#L617), ligne 617, `public void loadIni`.

<a id="f-cfg-005"></a>

### F-CFG-005 — Options de démarrage

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Préserver les intentions de h/v/w/l/r/n/s et le chemin de projet en argument, avec interface moderne documentée.
- Recette : L’aide liste les options; ouvrir un fichier en argument charge le bon projet; les logs vont au chemin annoncé.
- Composant : `platform`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-PROJ-001.
- Preuve : [_archive/filius-master/src/main/java/filius/rahmenprogramm/FiliusArgs.java](../../_archive/filius-master/src/main/java/filius/rahmenprogramm/FiliusArgs.java#L28), ligne 28, `options.addOption`.

<a id="f-cfg-006"></a>

### F-CFG-006 — Journaux de diagnostic

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Séparer journal technique, trace réseau et données pédagogiques; permettre un export de diagnostic.
- Recette : Le journal situe une erreur sans inclure par défaut les mots de passe et contenus mail.
- Composant : `platform`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-PROJ-001.
- Preuve : [_archive/filius-master/src/main/java/filius/rahmenprogramm/TeeOutputStream.java](../../_archive/filius-master/src/main/java/filius/rahmenprogramm/TeeOutputStream.java#L39), ligne 39, `public class TeeOutputStream`.

<a id="f-cfg-007"></a>

### F-CFG-007 — Installation et association des fichiers

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Distribuer une application desktop autonome et associer le format projet sur les plateformes ciblées.
- Recette : Un utilisateur ouvre un projet depuis le gestionnaire de fichiers et désinstalle sans supprimer ses projets.
- Composant : `platform`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-PROJ-001.
- Preuve : [_archive/filius-master/Readme.md](../../_archive/filius-master/Readme.md#L23), ligne 23, `## Installation`.

<a id="f-cfg-008"></a>

### F-CFG-008 — Scénarios et exercices fournis

- Origine : `filius`; état du référentiel : `present`.
- Exigence : Recréer les exemples de connexion directe, LAN, routage, DHCP, DNS, web, mail, NAT, pare-feu, ARP, P2P et modem.
- Recette : Chaque famille possède un exemple documenté et une recette reproductible.
- Composant : `platform`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : F-PROJ-001.
- Preuve : [_archive/filius-master/src/info/Introduction_Filius.md](../../_archive/filius-master/src/info/Introduction_Filius.md#L349), ligne 349, `## Exercises: Design and Simulate Networks`.


<a id="nfr"></a>

## NFR — Qualité et contraintes transversales

<a id="f-nfr-001"></a>

### F-NFR-001 — Simulation hors ligne

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Exécuter tous les réseaux simulés sans accès Internet ni privilèges système.
- Recette : Les recettes LAN, DNS et HTTP réussissent réseau réel coupé; aucun port réel n’est ouvert.
- Composant : `quality`; jalon : `J0`; état : `in_progress`.
- Prérequis fonctionnels : aucun.

<a id="f-nfr-002"></a>

### F-NFR-002 — Contrats indépendants du langage

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Versionner données, commandes, événements et erreurs sans types propres à une implémentation.
- Recette : Deux adaptateurs consomment les mêmes fixtures sans traduction de classes métier.
- Composant : `quality`; jalon : `J0`; état : `verified`.
- Prérequis fonctionnels : aucun.

<a id="f-nfr-003"></a>

### F-NFR-003 — Exécution sans interface graphique

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Exécuter scénarios et assertions par une entrée automatisable.
- Recette : Un scénario peut être lancé avec graine et borne virtuelle et produire un code de sortie et une trace.
- Composant : `quality`; jalon : `J0`; état : `verified`.
- Prérequis fonctionnels : aucun.

<a id="f-nfr-004"></a>

### F-NFR-004 — Limites et réactivité

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Borner files, traces, tailles de projet et traitements; mesurer les coûts du moteur et de l’UI.
- Recette : Le benchmark de référence publie machine, charge, latence UI et mémoire; dépasser une limite produit une erreur lisible.
- Composant : `quality`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-nfr-005"></a>

### F-NFR-005 — Accessibilité

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Prévoir clavier, focus visible, contrastes, zoom texte et alternative aux seules couleurs.
- Recette : Le parcours créer/configurer/relier/lancer/inspecter s’effectue au clavier; les états restent compréhensibles sans couleur.
- Composant : `quality`; jalon : `J1`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-nfr-006"></a>

### F-NFR-006 — Import et rendu maîtrisés

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Limiter archives, chemins, XML et HTML; séparer contenu simulé et accès hôte.
- Recette : Des archives à chemins sortants, XML à entités externes et pages tentant une requête réelle sont rejetées.
- Composant : `quality`; jalon : `J0`; état : `in_progress`.
- Prérequis fonctionnels : aucun.

<a id="f-nfr-007"></a>

### F-NFR-007 — Plateformes desktop

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Cibler Windows, Linux et macOS avec recettes de packaging et de rendu par plateforme.
- Recette : Le parcours réseau minimal et la sauvegarde fonctionnent sur les trois systèmes via une matrice de validation.
- Composant : `quality`; jalon : `J7`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-nfr-008"></a>

### F-NFR-008 — Traçabilité des changements

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Relier exigences, tâches d’implémentation, preuves de recette et décisions.
- Recette : Aucune exigence ne peut passer à verified sans preuve reproductible ni tâche associée.
- Composant : `quality`; jalon : `J0`; état : `verified`.
- Prérequis fonctionnels : aucun.

<a id="f-nfr-009"></a>

### F-NFR-009 — Provenance et licences

- Origine : `modernisation`; état du référentiel : `absent_du_referentiel`.
- Exigence : Conserver provenance des références et des actifs, et décider explicitement la politique de réutilisation.
- Recette : Une décision documente les éléments repris et leurs notices avant toute redistribution.
- Composant : `quality`; jalon : `J0`; état : `verified`.
- Prérequis fonctionnels : aucun.


<a id="adv"></a>

## ADV — Évolutions hors parité

<a id="f-adv-001"></a>

### F-ADV-001 — IPv6

- Origine : `extension_optionnelle`; état du référentiel : `absent_du_referentiel`.
- Exigence : Étendre le modèle aux adresses IPv6, NDP et ICMPv6 dans un lot distinct.
- Recette : Un LAN double pile résout ses voisins et échange des pings IPv6.
- Composant : `future`; jalon : `J9`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-adv-002"></a>

### F-ADV-002 — VLAN et prévention normalisée des boucles

- Origine : `extension_optionnelle`; état du référentiel : `absent_du_referentiel`.
- Exigence : Ajouter VLAN, trunks et protocole de prévention des boucles avec contrats propres.
- Recette : Deux VLAN restent isolés et une boucle est traitée par le protocole retenu.
- Composant : `future`; jalon : `J9`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-adv-003"></a>

### F-ADV-003 — Protocoles et sécurité avancés

- Origine : `extension_optionnelle`; état du référentiel : `absent_du_referentiel`.
- Exigence : Étudier TLS, DNS supplémentaires, routage avancé et suivi de connexion réel.
- Recette : Chaque protocole ajouté possède sa matrice de conformité et un scénario pédagogique dédié.
- Composant : `future`; jalon : `J9`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-adv-004"></a>

### F-ADV-004 — Exercices évaluables

- Origine : `extension_optionnelle`; état du référentiel : `absent_du_referentiel`.
- Exigence : Décrire objectifs, états attendus, feedback et corrections exportables.
- Recette : Un exercice valide automatiquement une panne DNS réparée et explique un échec.
- Composant : `future`; jalon : `J9`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-adv-005"></a>

### F-ADV-005 — Collaboration synchronisée

- Origine : `extension_optionnelle`; état du référentiel : `absent_du_referentiel`.
- Exigence : Étudier l’édition partagée et les sessions de simulation synchronisées.
- Recette : Deux éditeurs convergent sur le même projet avec conflits explicités.
- Composant : `future`; jalon : `J9`; état : `not_started`.
- Prérequis fonctionnels : aucun.

<a id="f-adv-006"></a>

### F-ADV-006 — Exécution dans le navigateur

- Origine : `extension_optionnelle`; état du référentiel : `absent_du_referentiel`.
- Exigence : Étudier un adaptateur web du moteur et ses restrictions.
- Recette : Un scénario de référence donne la même trace que l’adaptateur desktop.
- Composant : `future`; jalon : `J9`; état : `not_started`.
- Prérequis fonctionnels : aucun.
