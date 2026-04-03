// Store des labels de disques : couleur hex et surnom personnalisé par chemin de montage.
// Charge les labels connus au démarrage via invoke('get_disk_label') pour chaque disque détecté.
// Expose getDiskLabelFor(path) et saveDiskLabelFor(label) pour la sidebar.

import {
  getDiskLabel as apiGetDiskLabel,
  saveDiskLabel as apiSaveDiskLabel,
  type DiskLabel,
} from '$lib/tauri'

// ── State ─────────────────────────────────────────────────────────────────────

let labels = $state<Record<string, DiskLabel>>({})

// ── Public API ────────────────────────────────────────────────────────────────

/** Load the disk label for a single volume path (no-op if already loaded). */
export async function loadDiskLabel(path: string): Promise<void> {
  if (path in labels) return
  const label = await apiGetDiskLabel(path)
  if (label) labels = { ...labels, [path]: label }
}

/** Return the stored label for `path`, or null if none has been set. */
export function getDiskLabelFor(path: string): DiskLabel | null {
  return labels[path] ?? null
}

/** Persist a label and update the store immediately. */
export async function saveDiskLabelFor(label: DiskLabel): Promise<void> {
  await apiSaveDiskLabel(label)
  labels = { ...labels, [label.path]: label }
}
