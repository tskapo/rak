<!-- TagManager.svelte : interface CRUD complète pour les tags globaux.
     Accessible via "Gérer les tags…" dans TagPicker ou les préférences.
     Permet de créer un tag (nom + couleur), renommer, recolorer et supprimer.
     La suppression déclenche ON DELETE CASCADE côté SQLite → retire le tag de tous les fichiers. -->
