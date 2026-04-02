# Räk

> **Räk** (რა̈კ) — mot svane signifiant *"clignement d'œil"*.
> Un explorateur de fichiers desktop ultra-rapide pour les partages réseau SMB/NFS.

Finder et Explorer font des requêtes bloquantes sur SMB — tout se fige.
Räk résout ce problème avec un cache SQLite local alimenté en arrière-plan par Rust,
pendant que l'interface Svelte lit depuis ce cache et reste toujours réactive.

---

## Prérequis

| Outil | Version minimale |
|---|---|
| [Node.js](https://nodejs.org) | 18+ |
| [Rust](https://rustup.rs) | stable (1.77+) |
| [Bun](https://bun.sh) | 1.0+ |
| [Tauri CLI](https://tauri.app/start/prerequisites/) | 2.x |

Sur macOS, installez également les outils de ligne de commande Xcode :

```bash
xcode-select --install
```

---

## Installation

```bash
# 1. Cloner le dépôt
git clone <url-du-depot>
cd räk

# 2. Installer les dépendances JavaScript
bun install
```

---

## Lancer en mode développement

```bash
bun run tauri dev
```

L'application s'ouvre avec le rechargement à chaud activé (frontend et backend).

---

## Construire pour la production

```bash
bun run tauri build
```

L'installeur natif (.dmg sur macOS, .msi sur Windows, .AppImage sur Linux)
est généré dans `src-tauri/target/release/bundle/`.

---

## Comment fonctionne l'indexation

Räk ne lit **jamais** directement depuis le partage réseau pour afficher la liste des fichiers.
À la place, il maintient un cache local SQLite (`rak.db`) et l'affiche instantanément.

### Étape 1 — Démarrage de l'application

Au lancement, Räk détermine automatiquement le dossier à indexer en priorité :

1. Le **dernier dossier visité** (si le chemin est encore accessible sur le réseau).
2. Le **dossier personnel** de l'utilisateur (`$HOME`) si aucun récent n'est disponible.

Un scan de ce dossier démarre immédiatement **en arrière-plan** — l'interface est
utilisable sans attendre la fin du scan.

### Étape 2 — Scan en arrière-plan

Le service `IndexerService` (Rust / tokio) parcourt l'arborescence de façon asynchrone :

- Pour chaque fichier : nom, taille, date de modification, type MIME (détecté sur les 36 premiers octets).
- Pour chaque dossier : ajout récursif à la file d'attente de scan.
- Les entrées sont écrites dans la table `file_cache` (SQLite) via un **upsert** atomique.
- En cas d'erreur de lecture (perte réseau, droits insuffisants), l'entrée est simplement ignorée — le scan continue.

### Étape 3 — Événements de progression

Pendant le scan, Räk émet l'événement Tauri `indexer:progress` après chaque dossier traité.
Le frontend peut s'y abonner pour afficher un indicateur de progression.

```ts
// Exemple d'écoute côté Svelte
import { listen } from '@tauri-apps/api/event'

await listen('indexer:progress', (event) => {
  console.log(event.payload.path, event.payload.filesIndexed)
})
```

### Étape 4 — Navigation

Quand l'utilisateur ouvre un dossier, Räk lit depuis le cache SQLite — **aucune requête réseau**.
Si le cache est vide ou périmé, un nouveau scan ciblé est déclenché en arrière-plan.

### Étape 5 — Surveillance en temps réel (WatcherService)

Le `WatcherService` (via la crate `notify`) surveille le système de fichiers.
Dès qu'un fichier est créé, modifié ou supprimé, le cache est invalidé et un
re-scan ciblé du dossier concerné est déclenché automatiquement.

---

## Structure du projet

```
src/                    Interface Svelte 5 (SvelteKit, UnoCSS)
src-tauri/src/
  commands/             Commandes Tauri exposées au frontend (IPC)
  services/
    cache.rs            Couche d'accès SQLite (sqlx)
    indexer.rs          Scanner asynchrone de répertoires (tokio)
    watcher.rs          Surveillance temps réel du filesystem (notify)
  db/schema.sql         Schéma SQLite + migrations
```

---

## Plateformes supportées

| OS | Chemin de montage SMB/NFS typique |
|---|---|
| macOS | `/Volumes/NAS` |
| Windows | `Z:\` ou `\\serveur\partage` |
| Linux | `/mnt/nas` |

L'application ne monte pas elle-même le partage — elle lit depuis un chemin
déjà monté par le système d'exploitation.

---

## Licence

MIT
