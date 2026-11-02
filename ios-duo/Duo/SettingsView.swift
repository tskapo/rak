import SwiftUI

struct SettingsView: View {
    @EnvironmentObject private var store: AccountStore
    @EnvironmentObject private var sessions: SessionManager
    @Environment(\.dismiss) private var dismiss
    @State private var accountToDelete: Account?

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    ForEach($store.accounts) { $account in
                        NavigationLink {
                            AccountDetailView(account: $account)
                        } label: {
                            HStack(spacing: 10) {
                                Circle().fill(account.color.color).frame(width: 12, height: 12)
                                Text(account.name)
                            }
                        }
                        .swipeActions {
                            if store.accounts.count > 1 {
                                Button("Supprimer", role: .destructive) {
                                    accountToDelete = account
                                }
                            }
                        }
                    }

                    Button {
                        store.add()
                    } label: {
                        Label("Ajouter un compte", systemImage: "plus.circle")
                    }
                } header: {
                    Text("Comptes")
                } footer: {
                    Text("Chaque compte a son propre espace isolé (cookies, stockage, cache) : tu restes connecté aux deux en même temps.")
                }

                Section("Connexion") {
                    Label("Connecte-toi avec ton adresse e-mail. « Continuer avec Google » est bloqué par Google dans les apps de ce type.", systemImage: "envelope")
                        .font(.callout)
                    Label("Reçu un lien de connexion par e-mail ? Ne l'ouvre pas (il partirait dans Safari) : appui long → « Copier », puis menu ⋯ → « Ouvrir le lien copié » sur le bon compte. Si un code est proposé, saisis-le directement.", systemImage: "link")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
            }
            .navigationTitle("Réglages")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("OK") { dismiss() }
                }
            }
            .confirmationDialog(
                "Supprimer « \(accountToDelete?.name ?? "") » ?",
                isPresented: Binding(
                    get: { accountToDelete != nil },
                    set: { if !$0 { accountToDelete = nil } }
                ),
                titleVisibility: .visible
            ) {
                Button("Supprimer le compte et ses données", role: .destructive) {
                    if let account = accountToDelete {
                        store.remove(account.id)
                        Task { await sessions.discard(account.id) }
                    }
                    accountToDelete = nil
                }
            } message: {
                Text("Tu seras déconnecté de ce compte dans l'app. Ton compte Claude lui-même n'est pas touché.")
            }
        }
    }
}

struct AccountDetailView: View {
    @Binding var account: Account
    @EnvironmentObject private var sessions: SessionManager
    @State private var confirmLogout = false
    @State private var isClearing = false

    var body: some View {
        Form {
            Section("Nom") {
                TextField("Nom du compte", text: $account.name)
            }

            Section("Couleur") {
                Picker("Couleur", selection: $account.color) {
                    ForEach(AccountColor.allCases) { color in
                        HStack {
                            Circle().fill(color.color).frame(width: 12, height: 12)
                            Text(color.label)
                        }
                        .tag(color)
                    }
                }
                .pickerStyle(.inline)
                .labelsHidden()
            }

            Section {
                Button(role: .destructive) {
                    confirmLogout = true
                } label: {
                    HStack {
                        Text("Se déconnecter et effacer les données")
                        if isClearing {
                            Spacer()
                            ProgressView()
                        }
                    }
                }
                .disabled(isClearing)
            } footer: {
                Text("Efface les cookies et le stockage de ce compte uniquement. L'autre compte reste connecté.")
            }
        }
        .navigationTitle(account.name)
        .confirmationDialog("Se déconnecter de « \(account.name) » ?", isPresented: $confirmLogout, titleVisibility: .visible) {
            Button("Se déconnecter", role: .destructive) {
                isClearing = true
                Task {
                    await sessions.session(for: account.id).clearData()
                    isClearing = false
                }
            }
        }
    }
}
