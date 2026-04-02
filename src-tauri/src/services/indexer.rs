// IndexerService : scanne un dossier de façon asynchrone (tokio) et alimente le cache SQLite.
// Appelé au démarrage de l'app et sur demande explicite (ex : dossier non encore indexé).
// Émet des événements Tauri pour notifier le frontend de la progression.
