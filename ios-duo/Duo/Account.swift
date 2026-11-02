import SwiftUI

/// Couleur associée à un compte, pour savoir d'un coup d'œil sur lequel on est.
enum AccountColor: String, Codable, CaseIterable, Identifiable {
    case orange, blue, green, purple, pink, gray

    var id: String { rawValue }

    var color: Color {
        switch self {
        case .orange: return Color(red: 0.85, green: 0.47, blue: 0.34)
        case .blue: return Color(red: 0.23, green: 0.51, blue: 0.96)
        case .green: return Color(red: 0.13, green: 0.65, blue: 0.42)
        case .purple: return Color(red: 0.55, green: 0.36, blue: 0.96)
        case .pink: return Color(red: 0.93, green: 0.29, blue: 0.60)
        case .gray: return Color(red: 0.45, green: 0.48, blue: 0.53)
        }
    }

    var label: String {
        switch self {
        case .orange: return "Orange"
        case .blue: return "Bleu"
        case .green: return "Vert"
        case .purple: return "Violet"
        case .pink: return "Rose"
        case .gray: return "Gris"
        }
    }
}

struct Account: Identifiable, Codable, Hashable {
    /// Sert aussi d'identifiant au `WKWebsiteDataStore` isolé du compte :
    /// il ne doit jamais changer, sinon la session est perdue.
    let id: UUID
    var name: String
    var color: AccountColor
}

/// Liste des comptes, sauvegardée dans UserDefaults.
final class AccountStore: ObservableObject {
    private static let storageKey = "duo.accounts"

    @Published var accounts: [Account] {
        didSet { save() }
    }

    init() {
        if let data = UserDefaults.standard.data(forKey: Self.storageKey),
           let saved = try? JSONDecoder().decode([Account].self, from: data),
           !saved.isEmpty {
            accounts = saved
        } else {
            accounts = [
                Account(id: UUID(), name: "Perso", color: .orange),
                Account(id: UUID(), name: "Pro", color: .blue),
            ]
            save()
        }
    }

    func add() {
        let used = Set(accounts.map(\.color))
        let color = AccountColor.allCases.first { !used.contains($0) } ?? .gray
        accounts.append(Account(id: UUID(), name: "Compte \(accounts.count + 1)", color: color))
    }

    func remove(_ id: UUID) {
        guard accounts.count > 1 else { return }
        accounts.removeAll { $0.id == id }
    }

    private func save() {
        if let data = try? JSONEncoder().encode(accounts) {
            UserDefaults.standard.set(data, forKey: Self.storageKey)
        }
    }
}
