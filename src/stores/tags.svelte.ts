// Store des tags globaux et des associations fichier ↔ tags.
// Charge la liste complète des tags via invoke('get_tags') au démarrage.
// Expose createTag(), deleteTag(), getFileTags(path), setFileTags(path, tagIds).
// Les mutations se reflètent immédiatement dans le store ($state) sans re-fetch.
