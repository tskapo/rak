// WatcherService : surveille le système de fichiers via la crate `notify`.
// Invalide les entrées du cache SQLite lorsqu'un fichier est créé, modifié ou supprimé.
// Déclenche un re-scan ciblé du dossier concerné via IndexerService.
