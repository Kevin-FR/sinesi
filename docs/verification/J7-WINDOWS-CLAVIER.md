# Qualification partielle J7 — recette clavier de la fenêtre Windows

F-CFG-007 et F-NFR-007 dans RS-PLATFORM-J7 et RS-QUALITY-J7. Les
prérequis F-PROJ-001 et RS-PROJECTS-J1 sont vérifiés.

Le mode `-Keyboard` lance l’application installée par son ProgID dans une
session de test interactive. Pour chaque combinaison FR EN ES IT PT DE ×
Icônes/Schéma, il vérifie le profil effectif à 150 %, avance avec Tab et
observe le focus UI Automation avant Entrée. Il ajoute un portable,
sauvegarde avec Ctrl+S, ouvre la simulation avec F3, envoie un ping et
cherche `icmp.echo.reply` dans les échanges. Le premier plan doit rester
la fenêtre du processus associé pendant la séquence. La recette compare
les empreintes des fichiers d’exercice avant et après la désinstallation.
[UI Automation](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.automationelement.focusedelement) ·
[SendKeys](https://learn.microsoft.com/en-us/dotnet/api/system.windows.forms.sendkeys.sendwait) ·
[Fenêtre de premier plan](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow)

## Contrôles exécutés

| Contrôle | Résultat et environnement |
| --- | --- |
| PowerShell 7.4.20 sur macOS arm64 | Les deux scripts s’analysent. Le C# de ShellExecuteEx et de la lecture du premier plan compile. Les douze profils écrits en UTF-8 sans BOM sont relus à 150 %. La collecte et la relecture des empreintes couvrent 24 fichiers. |
| Interfaces simulées sous PowerShell | La séquence suit Tab, Entrée, Ctrl+S, F3 et les contrôles réseau. Un changement synthétique du premier plan arrête la séquence après Tab, avant Entrée. |
| Windows PE GNU x64 sous CrossOver | Le binaire `sns-desktop.exe` relit chacun des douze profils et le projet Unicode à quatre équipements en mode `--check-config`. |
| Workflow | L’option booléenne `windows_keyboard` est désactivée par défaut et appelle Windows PowerShell. |

L’observation réelle de l’arbre UI Automation, l’envoi des touches et
l’essai depuis l’Explorateur restent à effectuer sur Windows. Les contrôles
locaux n’utilisent aucune fenêtre native Windows. Le passage par Finder
et le parcours clavier macOS restent également ouverts.

## Exécution native prévue

Depuis la racine du dépôt, dans une session Windows interactive de test
avec l’installateur x64 et un nouveau dossier de travail :

```powershell
powershell.exe -NoProfile -STA -File packaging\smoke-windows.ps1 `
    -Package artifacts\packages\Sinesi-0.1.0-windows-x64.exe `
    -WorkDirectory artifacts\windows-native-qualification -Keyboard
```

L’option `windows_keyboard` active la même recette dans le workflow
de paquets. La preuve doit conserver le journal des douze parcours
et la confirmation du retrait. L’ouverture depuis l’Explorateur
complète la recette basée sur la sélection explicite du ProgID.

[Contrôles PowerShell et simulation](j7-windows-keyboard-local.log) ·
[Lecture Windows sous CrossOver](j7-windows-keyboard-config.log) ·
[Manifeste](j7-windows-keyboard-code-manifest.json) ·
[Reconstruction](j7-windows-keyboard-source-patch.json)
