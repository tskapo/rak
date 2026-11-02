# Duo — deux comptes Claude sur iPhone

Petite app iOS personnelle qui ouvre **claude.ai** (chat **et** Claude Code) dans
plusieurs WebViews **isolées** : chaque compte a ses propres cookies, son stockage et
son cache (`WKWebsiteDataStore(forIdentifier:)`, iOS 17+). Les deux comptes restent
connectés en même temps ; on passe de l'un à l'autre d'un tap.

## Fonctions

- Comptes « Perso » et « Pro » par défaut, renommables, avec une couleur chacun
  (barre du haut teintée pour éviter de se tromper). On peut en ajouter d'autres.
- Bascule instantanée : toutes les sessions restent chargées en arrière-plan.
- Menu ⋯ : Chat, Claude Code (`claude.ai/code`), page suivante, recharger,
  copier le lien, ouvrir un lien copié, réglages.
- Liens externes et fenêtres OAuth (GitHub, connecteurs…) ouverts dans une feuille
  qui garde le compte courant ; bouton pour basculer dans Safari.
- Envoi de fichiers/photos, micro (dictée), alert/confirm JavaScript.
- Déconnexion par compte (efface uniquement les données de ce compte).

## Installer sur ton iPhone

Il faut un **Mac avec Xcode 15+**.

```bash
brew install xcodegen
cd ios-duo
xcodegen            # génère Duo.xcodeproj
open Duo.xcodeproj
```

Dans Xcode :
1. Cible **Duo** → *Signing & Capabilities* → choisis ton **Team** (ton Apple ID suffit).
   Change le *Bundle Identifier* si Xcode dit qu'il est déjà pris.
2. Branche l'iPhone, sélectionne-le, puis ▶︎ *Run*.
3. Sur l'iPhone : Réglages → Général → VPN et gestion de l'appareil → faire confiance
   au développeur. (iOS 16+ : activer aussi le *Mode développeur*.)

> Avec un Apple ID gratuit, l'app doit être réinstallée depuis Xcode tous les **7 jours**.
> Avec un compte Apple Developer (99 €/an), c'est un an, et TestFlight devient possible.

## Se connecter

- Utilise **l'adresse e-mail** de chaque compte. « Continuer avec Google » ne marche
  pas : Google refuse les connexions dans les WebViews intégrées.
- Si tu reçois un **lien** par e-mail, ne l'ouvre pas (il irait dans Safari) :
  appui long → *Copier*, puis dans Duo, sur le bon compte, ⋯ → **Ouvrir le lien copié**.
  Si un **code** est proposé, saisis-le directement.

## Limites

- Pas de notifications push (c'est une enveloppe web, pas l'app officielle).
- Usage personnel : l'app affiche simplement le site officiel avec tes propres comptes.
  Ne la publie pas sur l'App Store avec le nom ou le logo Claude.
- Pas d'icône d'app fournie : ajoute un `Assets.xcassets` avec un AppIcon si tu veux.

## Structure

| Fichier | Rôle |
|---|---|
| `Duo/DuoApp.swift` | Point d'entrée |
| `Duo/Account.swift` | Modèle de compte + sauvegarde (UserDefaults) |
| `Duo/WebSession.swift` | WebView isolée par compte, navigation, permissions |
| `Duo/PopupSession.swift` | Fenêtres secondaires (OAuth, liens), dialogues JS |
| `Duo/RootView.swift` | Barre de comptes, menu, affichage des WebViews |
| `Duo/SettingsView.swift` | Réglages des comptes, déconnexion |
