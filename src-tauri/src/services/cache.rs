// CacheService : couche d'accès à la base SQLite via sqlx.
// Expose des fonctions async pour lire/écrire dans file_cache, folder_prefs et recents.
// Encapsule toute la logique SQL — les commandes et services ne manipulent jamais sqlx directement.
