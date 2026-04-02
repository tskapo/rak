# Räk — CONTEXT.md

> **Räk** (რა̈კ) — mot svane signifiant *"clignement d'œil"* (un instant, une fraction de seconde).
> Le nom capture la promesse de l'app : voir ses fichiers distants en un éclair.

## Vue d'ensemble

**Räk** est un file browser desktop multiplateforme (macOS / Windows / Linux) conçu pour accélérer l'accès aux partages réseau SMB/NFS.
Le problème central : Finder et Explorer font des requêtes bloquantes sur SMB → tout freezes.
La solution : cache local SQLite alimenté en arrière-plan par Rust, UI Svelte qui lit depuis ce cache.

---

## Stack

### Frontend
- **Svelte 5** (runes — réactivité compile-time)
- **SvelteKit** en mode static (`adapter-static`)
- **UnoCSS** pour les styles (Tailwind-compatible)
- **Lucide Svelte** pour les icônes
- **@tauri-apps/api** pour le bridge IPC vers Rust
- Virtual scroll **custom** (`src/lib/VirtualList.svelte`, hauteur fixe)
- i18n **custom** (`src/lib/i18n.svelte.ts`, store Svelte 5 runes)

### Backend
- **Tauri v2** (Rust)
- **tokio** — async runtime
- **sqlx + SQLite** — cache local (metadata, préférences dossiers, historique)
- **notify** — watch filesystem en temps réel
- **image** — génération de thumbnails
- **infer** — détection de type MIME

---

## Architecture

```
src-tauri/
├─ src/
│   ├─ main.rs
│   ├─ commands/
│   │   ├─ fs.rs          # list_dir, get_metadata, read_file_chunk
│   │   ├─ preview.rs     # generate_preview (image/pdf/video)
│   │   ├─ prefs.rs       # get_folder_prefs, save_folder_prefs
│   │   └─ recent.rs      # get_recents, add_recent
│   ├─ services/
│   │   ├─ indexer.rs     # scan async + alimentation SQLite
│   │   ├─ watcher.rs     # notify → invalidation cache
│   │   └─ cache.rs       # accès SQLite (sqlx)
│   └─ db/
│       └─ schema.sql     # migrations SQLite

src/
├─ lib/
│   ├─ VirtualList.svelte  # scroll virtuel custom (hauteur fixe)
│   ├─ i18n.svelte.ts      # store i18n custom (runes)
│   ├─ locales/
│   │   ├─ fr.json
│   │   └─ en.json
│   └─ tauri.ts            # wrappers typés autour de invoke()
├─ stores/
│   ├─ navigation.svelte.ts   # dossier courant, historique, breadcrumb
│   ├─ folderPrefs.svelte.ts  # vue par dossier (list/column/detail)
│   ├─ theme.svelte.ts        # thème actif (light/dark/system)
│   └─ files.svelte.ts        # contenu dossier courant (cache réactif)
├─ components/
│   ├─ layout/
│   │   ├─ AppShell.svelte
│   │   └─ SidebarPane.svelte     # favoris + arborescence
│   ├─ browser/
│   │   ├─ BrowserView.svelte     # vue principale
│   │   ├─ FileListList.svelte    # vue liste + VirtualList
│   │   ├─ FileListColumn.svelte  # vue colonne style Finder
│   │   ├─ FileListDetail.svelte  # vue détail (nom, taille, date, type)
│   │   ├─ FileItem.svelte        # item générique (utilisé par les 3 vues)
│   │   └─ ViewToggle.svelte      # boutons list/column/detail
│   ├─ preview/
│   │   └─ PreviewPanel.svelte    # spacebar → aperçu fit-to-screen
│   └─ ui/
│       ├─ Breadcrumb.svelte
│       └─ RecentList.svelte
└─ routes/
    ├─ +layout.svelte
    ├─ +page.svelte           # → redirect vers /browser
    └─ browser/
        └─ +page.svelte
```

---

## Modèles de données

### SQLite — schema

```sql
-- Metadata fichiers (cache)
CREATE TABLE file_cache (
  path        TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  is_dir      BOOLEAN NOT NULL,
  size        INTEGER,
  modified_at INTEGER,
  mime_type   TEXT,
  indexed_at  INTEGER NOT NULL
);

-- Préférences par dossier
CREATE TABLE folder_prefs (
  path       TEXT PRIMARY KEY,
  view_mode  TEXT NOT NULL DEFAULT 'list',  -- 'list' | 'column' | 'detail'
  sort_by    TEXT NOT NULL DEFAULT 'name',  -- 'name' | 'date' | 'size'
  sort_dir   TEXT NOT NULL DEFAULT 'asc',   -- 'asc' | 'desc'
  updated_at INTEGER NOT NULL
);

-- Historique récents
CREATE TABLE recents (
  path        TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  is_dir      BOOLEAN NOT NULL,
  visited_at  INTEGER NOT NULL
);

-- Préférences globales de l'application (clé/valeur)
-- Clés réservées : 'theme' ('light'|'dark'|'system'), 'locale' ('fr'|'en')
-- Stockées en SQLite — indépendantes de l'OS et de localStorage
CREATE TABLE app_prefs (
  key        TEXT PRIMARY KEY,
  value      TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

-- Couleur et nom personnalisé par disque/partage monté
CREATE TABLE disk_labels (
  path        TEXT PRIMARY KEY,  -- chemin de montage (ex: /Volumes/NAS)
  color       TEXT NOT NULL,     -- couleur hex (ex: '#4a90d9')
  custom_name TEXT,              -- surnom optionnel (ex: 'NAS Maison')
  updated_at  INTEGER NOT NULL
);

-- Tags définis par l'utilisateur (palettes de couleurs libres)
CREATE TABLE tags (
  id    INTEGER PRIMARY KEY AUTOINCREMENT,
  name  TEXT NOT NULL UNIQUE,
  color TEXT NOT NULL  -- couleur hex
);

-- Association fichiers/dossiers ↔ tags (many-to-many)
CREATE TABLE file_tags (
  file_path TEXT NOT NULL,
  tag_id    INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY (file_path, tag_id)
);
```

### TypeScript — types partagés

```ts
interface FileEntry {
  path: string
  name: string
  isDir: boolean
  size?: number
  modifiedAt?: number
  mimeType?: string
}

interface FolderPrefs {
  path: string
  viewMode: 'list' | 'column' | 'detail'
  sortBy: 'name' | 'date' | 'size'
  sortDir: 'asc' | 'desc'
}

interface AppPrefs {
  theme: 'light' | 'dark' | 'system'
  locale: Locale
}

interface DiskLabel {
  path: string
  color: string        // hex, ex: '#4a90d9'
  customName?: string  // surnom affiché dans la sidebar
}

interface Tag {
  id: number
  name: string
  color: string        // hex
}

type Locale = 'fr' | 'en' | 'ka'
```

---

## Commandes Tauri (IPC)

```ts
// src/lib/tauri.ts
invoke('list_dir',          { path: string })        → FileEntry[]
invoke('get_metadata',      { path: string })        → FileEntry
invoke('generate_preview',  { path: string })        → string  // base64 ou chemin tmp
invoke('get_folder_prefs',  { path: string })        → FolderPrefs
invoke('save_folder_prefs', { prefs: FolderPrefs })  → void
invoke('get_recents',       { limit: number })       → FileEntry[]
invoke('add_recent',        { entry: FileEntry })    → void

// Préférences globales (thème, langue)
invoke('get_app_prefs')                              → AppPrefs
invoke('save_app_pref',     { key, value: string })  → void

// Labels de disques
invoke('get_disk_label',    { path: string })        → DiskLabel | null
invoke('save_disk_label',   { label: DiskLabel })    → void

// Tags
invoke('get_tags')                                   → Tag[]
invoke('create_tag',        { name, color: string }) → Tag
invoke('delete_tag',        { id: number })          → void
invoke('get_file_tags',     { path: string })        → Tag[]
invoke('set_file_tags',     { path, tagIds: number[] }) → void
```

---

## Virtual scroll custom

Hauteur fixe par item :
- Vue liste : `40px`
- Vue détail : `40px`
- Vue colonne : hauteur variable par colonne (géré séparément)

```svelte
<!-- src/lib/VirtualList.svelte -->
<!-- Props: items[], itemHeight, containerHeight -->
<!-- Slot: { item, index } -->
```

---

## i18n custom

```ts
// src/lib/i18n.svelte.ts
// Rune $state pour locale active
// Fonction t(key: string): string  — dot notation (ex: t('browser.views.list'))
// Fonction setLocale(l: Locale): void
// Langues : 'fr' (défaut) | 'en' | 'ka' (géorgien)
```

---

## Comportements clés

### Navigation
- Cliquer sur un dossier → `navigationStore.navigate(path)` → `invoke('list_dir')`
- Chaque navigation sauvegarde les prefs du dossier courant avant de partir
- Breadcrumb cliquable pour remonter

### Preview (barre espace)
- `keydown Space` sur un item → ouvre `PreviewPanel`
- Le panneau occupe toute la hauteur disponible à droite (ou modal selon taille fenêtre)
- Images : fit-to-screen avec ratio préservé
- PDF : rendu page par page
- Vidéo : player HTML5 natif
- Autres : icône type + metadata

### Thèmes
- Light / Dark / System (via `matchMedia` pour le mode system)
- **Persisté dans SQLite** (`app_prefs.key = 'theme'`) via `appPrefsStore` — **pas localStorage**
- Initialisé au démarrage depuis SQLite avant le premier rendu pour éviter le flash
- **Entièrement piloté par variables CSS** — aucune couleur, taille ou ombre codée en dur dans les composants
- Toutes les valeurs visuelles (couleurs, espacements, rayons, ombres) sont des `var(--xxx)` définis dans un fichier `src/styles/tokens.css`
- Ajouter un thème = surcharger les tokens sur `[data-theme="nom"]`, sans toucher aux composants

### Langue
- Indépendante de la langue système (l'utilisateur peut vouloir l'app en français sur un macOS anglais)
- **Persistée dans SQLite** (`app_prefs.key = 'locale'`)
- Chargée au démarrage via `get_app_prefs` → passée à `i18n.svelte.ts` via `setLocale()`
- Défaut : `'fr'`
- Langues disponibles : Français (`fr`), English (`en`), ქართული (`ka`)
- Sélecteur : **dropdown** (`<select>` natif ou composant custom) — pas de pills

### Tags et couleurs par disque
- Chaque disque/partage affiché dans la sidebar peut recevoir : une couleur hex et un surnom
- Un fichier ou dossier peut avoir N tags (colored pills) définis librement par l'utilisateur
- Les tags sont globaux (partagés entre tous les disques), l'association est par chemin absolu
- **CRUD complet** : création, renommage, changement de couleur et suppression via `TagManager.svelte`
- Accès : clic droit → "Étiqueter..." → `TagPicker.svelte` (assigner/retirer) + lien "Gérer les tags…" → `TagManager.svelte`
- Suppression d'un tag : cascade automatique sur `file_tags` (SQLite `ON DELETE CASCADE`)
- Les tags sont affichés inline dans `FileItem.svelte` (vue liste et détail)

### Dimensions et échelle d'affichage
- **Toutes les dimensions sont en `rem`** — pas de `px` dans les styles (sauf bordures 1px et SVG)
- La valeur de `font-size` sur `<html>` est l'unique levier de mise à l'échelle : l'utilisateur peut la régler (ex: 14px → tout s'agrandit proportionnellement)
- Préférence d'échelle stockée dans `app_prefs` (`key = 'ui_scale'`, valeur float, défaut `1.0`)
- Appliquée au démarrage via `document.documentElement.style.fontSize = base * scale + 'px'`
- Le `VirtualList` recalcule `itemHeight` en px à partir de la valeur rem courante (`parseFloat(getComputedStyle(html).fontSize) * REM_VALUE`)

### État par dossier
- À chaque navigation, `folderPrefsStore` charge les prefs du nouveau dossier (`get_folder_prefs`)
- À chaque changement (vue, tri), `folderPrefsStore` persiste immédiatement (`save_folder_prefs`)
- L'état survit entre les sessions : rouvrir un dossier restaure exactement la dernière vue et le tri

### Récents
- Chaque dossier visité → ajouté dans `recents` SQLite
- Affiché dans la sidebar et dans une vue dédiée `/recent`
- Limité à 50 entrées (FIFO)

---

## Philosophie de code

> Le code est lu bien plus souvent qu'il n'est écrit. Chaque fichier doit être compréhensible par un humain qui le découvre pour la première fois.

### Principes fondamentaux

- **DRY** — Don't Repeat Yourself. Toute logique dupliquée devient une abstraction nommée
- **KISS** — Keep It Simple. La solution la plus simple qui fonctionne est la bonne
- **YAGNI** — You Aren't Gonna Need It. On n'anticipe pas des besoins hypothétiques

### Lisibilité avant tout

- Code **aéré** — lignes vides entre les blocs logiques, pas de code dense et compact
- Noms **explicites** — `loadFolderContents()` pas `load()`, `isDirectoryEmpty` pas `empty`
- **Pas de magie** — si une ligne nécessite un commentaire pour être comprise, elle doit être réécrite
- Pas d'abréviations sauf conventions universelles (`i`, `e`, `err`, `ctx`)

### Modules petits et focalisés

- **Un fichier = une responsabilité** — si on hésite à nommer un fichier, c'est qu'il fait trop de choses
- Taille cible : **< 150 lignes** par fichier, **< 50 lignes** par fonction
- Si un composant grossit → on l'éclate en sous-composants nommés
- **Pas de composant fourre-tout** — pas de `utils.ts` ou `helpers.ts` générique

### Documentation

- Chaque **fichier** commence par un commentaire de 1-3 lignes expliquant son rôle
- Chaque **fonction publique** a un JSDoc si son comportement n'est pas évident
- Chaque **commande Tauri** est documentée côté Rust (`///`) et côté TypeScript (JSDoc)
- Les **décisions d'architecture non-évidentes** sont expliquées en commentaire inline

```ts
// ✅ Bien
/** Charge le contenu d'un dossier depuis le cache SQLite.
 *  Si le cache est vide ou périmé, déclenche un re-scan en arrière-plan. */
async function loadFolderContents(path: string): Promise<FileEntry[]> { … }

// ❌ Mal
async function load(p: string) { … }
```

```svelte
<!-- ✅ Bien — un composant, une responsabilité -->
<!-- FileItem.svelte : affiche un seul fichier/dossier dans la liste -->

<!-- ❌ Mal -->
<!-- BrowserView.svelte : contient la liste, la sidebar, la preview, les prefs... -->
```

### Structure d'un composant Svelte

```svelte
<!-- 1. Commentaire de rôle (1-3 lignes) -->
<!-- 2. <script> : props → state → derived → fonctions → lifecycle -->
<!-- 3. Template : logique minimale, déléguer aux sous-composants -->
<!-- 4. <style> : scoped, variables CSS uniquement -->
```

---

## Conventions de code

- Svelte 5 : **toujours utiliser les runes** (`$state`, `$derived`, `$effect`, `$props`)
- Pas de `writable()` / `readable()` — ancienne API Svelte 4
- Nommage stores : `useCamelCase.svelte.ts`
- Nommage composants : `PascalCase.svelte`
- Les `invoke()` Tauri sont **toujours wrappés** dans `src/lib/tauri.ts` avec types stricts
- Gestion d'erreurs : `try/catch` sur tous les `invoke()`, erreurs affichées in-UI (pas de console.error silencieux)

---

## Plateformes cibles

| OS | SMB mount path | Notes |
|---|---|---|
| macOS | `/Volumes/NAS` | Monté via Finder ou `mount_smbfs` |
| Windows | `Z:\` ou `\\server\share` | Lecteur réseau mappé |
| Linux | `/mnt/nas` | Monté via `mount -t cifs` |

L'app ne monte pas elle-même le partage — elle lit depuis un chemin déjà monté par l'OS.

---

## À faire (backlog)

- [ ] Scaffolding projet Tauri + SvelteKit
- [ ] Schema SQLite + migrations (incl. app_prefs, disk_labels, tags, file_tags)
- [ ] `IndexerService` Rust (scan async)
- [ ] `WatcherService` Rust (notify)
- [ ] Commandes Tauri (list_dir, prefs, recents)
- [ ] Commandes Tauri (app_prefs, disk_labels, tags)
- [ ] `VirtualList.svelte` custom
- [ ] `i18n.svelte.ts` custom
- [ ] `appPrefsStore` (thème + langue persistés SQLite)
- [ ] `FileListList.svelte` (vue liste)
- [ ] `FileListDetail.svelte` (vue détail)
- [ ] `FileListColumn.svelte` (vue colonne)
- [ ] `FileItem.svelte` avec affichage tags inline
- [ ] `TagBadge.svelte` (pill coloré)
- [ ] `TagPicker.svelte` (popover assigner/retirer tags sur un fichier)
- [ ] `TagManager.svelte` (CRUD complet : créer, renommer, recolorer, supprimer un tag)
- [ ] `PreviewPanel.svelte` (spacebar)
- [ ] Sidebar + arborescence dossiers avec couleurs disques
- [ ] Système de thèmes via variables CSS (`src/styles/tokens.css`) — aucune couleur codée en dur
- [ ] Thèmes light/dark/system (persistance SQLite)
- [ ] Sélecteur de langue in-app (indépendant de l'OS)
- [ ] Mise à l'échelle UI (`ui_scale` dans app_prefs → `font-size` sur `<html>` en rem)
- [ ] `VirtualList.svelte` : recalcul `itemHeight` depuis rem courant
- [ ] Vue récents
