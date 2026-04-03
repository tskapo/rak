// Store des tags globaux et des associations fichier ↔ tags.
// Charge la liste complète des tags via get_tags au démarrage.
// Expose createTag(), deleteTag(), getFileTags(path), setFileTags(path, tagIds).
// Les mutations se reflètent immédiatement dans le store ($state) sans re-fetch.

import {
  getTags,
  createTag  as apiCreateTag,
  updateTag  as apiUpdateTag,
  deleteTag  as apiDeleteTag,
  getFileTags as apiGetFileTags,
  setFileTags as apiSetFileTags,
  type Tag,
} from '$lib/tauri'

// ── State ─────────────────────────────────────────────────────────────────────

let allTags  = $state<Tag[]>([])

/** Cache of file → tags associations, keyed by file path. */
let fileTags = $state<Record<string, Tag[]>>({})

// ── Public API ────────────────────────────────────────────────────────────────

/** Load all tags from SQLite. Call once at app startup. */
export async function loadTags(): Promise<void> {
  allTags = await getTags()
}

/** All user-defined tags, sorted alphabetically. */
export function getAllTags(): Tag[] { return allTags }

/** Create a new tag and add it to the store. */
export async function createTag(name: string, color: string): Promise<Tag> {
  const tag = await apiCreateTag(name, color)
  allTags   = [...allTags, tag].sort((a, b) => a.name.localeCompare(b.name))
  return tag
}

/** Update a tag's name and color, reflecting the change immediately in all caches. */
export async function updateTag(id: number, name: string, color: string): Promise<void> {
  await apiUpdateTag(id, name, color)
  allTags = allTags
    .map(t => t.id === id ? { ...t, name, color } : t)
    .sort((a, b) => a.name.localeCompare(b.name))
  fileTags = Object.fromEntries(
    Object.entries(fileTags).map(([path, tags]) => [
      path,
      tags.map(t => t.id === id ? { ...t, name, color } : t),
    ])
  )
}

/** Delete a tag by id, removing it from all cached file associations. */
export async function deleteTag(id: number): Promise<void> {
  await apiDeleteTag(id)
  allTags  = allTags.filter(t => t.id !== id)
  // Purge deleted tag from any cached file associations
  fileTags = Object.fromEntries(
    Object.entries(fileTags).map(([path, tags]) => [path, tags.filter(t => t.id !== id)])
  )
}

/**
 * Return the tags for `path`.
 * Fetches from SQLite on first call; subsequent calls use the in-memory cache.
 */
export async function getFileTags(path: string): Promise<Tag[]> {
  if (path in fileTags) return fileTags[path]

  const tags    = await apiGetFileTags(path)
  fileTags      = { ...fileTags, [path]: tags }
  return tags
}

/** Replace the tag set for `path` and update the cache immediately. */
export async function setFileTags(path: string, tagIds: number[]): Promise<void> {
  await apiSetFileTags(path, tagIds)
  fileTags = {
    ...fileTags,
    [path]: allTags.filter(t => tagIds.includes(t.id)),
  }
}

/** Evict a path from the tag cache (e.g. after a rename). */
export function evictFileTags(path: string): void {
  const { [path]: _, ...rest } = fileTags
  fileTags = rest
}
