import SwiftUI
import UIKit
import WebKit

struct RootView: View {
    @EnvironmentObject private var store: AccountStore
    @EnvironmentObject private var sessions: SessionManager
    @AppStorage("duo.selectedAccount") private var selectedID = ""
    @State private var showSettings = false

    private var selected: Account {
        store.accounts.first { $0.id.uuidString == selectedID } ?? store.accounts[0]
    }

    var body: some View {
        let current = selected
        VStack(spacing: 0) {
            TopBar(
                accounts: store.accounts,
                selected: current,
                session: sessions.session(for: current.id),
                onSelect: { selectedID = $0.id.uuidString },
                onSettings: { showSettings = true }
            )

            // Toutes les WebViews restent en vie : changer de compte est instantané
            // et chaque conversation garde son état.
            ZStack {
                ForEach(store.accounts) { account in
                    AccountPane(session: sessions.session(for: account.id), account: account)
                        .opacity(account.id == current.id ? 1 : 0)
                        .allowsHitTesting(account.id == current.id)
                        .accessibilityHidden(account.id != current.id)
                }
            }
            // Laisse WKWebView gérer lui-même le clavier.
            .ignoresSafeArea(.keyboard)
        }
        .sheet(isPresented: $showSettings) {
            SettingsView()
        }
    }
}

struct TopBar: View {
    let accounts: [Account]
    let selected: Account
    @ObservedObject var session: WebSession
    var onSelect: (Account) -> Void
    var onSettings: () -> Void

    var body: some View {
        let tint = selected.color.color
        HStack(spacing: 10) {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 6) {
                    ForEach(accounts) { account in
                        AccountChip(account: account, isSelected: account.id == selected.id) {
                            onSelect(account)
                        }
                    }
                }
            }

            Button {
                session.goBack()
            } label: {
                Image(systemName: "chevron.backward")
                    .font(.body.weight(.semibold))
                    .frame(width: 32, height: 32)
            }
            .disabled(!session.canGoBack)
            .accessibilityLabel("Retour")

            Menu {
                Button("Chat", systemImage: "bubble.left.and.bubble.right") { session.open(.chat) }
                Button("Claude Code", systemImage: "chevron.left.forwardslash.chevron.right") { session.open(.code) }
                Divider()
                Button("Page suivante", systemImage: "chevron.forward") { session.goForward() }
                    .disabled(!session.canGoForward)
                Button("Recharger", systemImage: "arrow.clockwise") { session.reload() }
                Button("Ouvrir le lien copié", systemImage: "doc.on.clipboard") { session.openPasteboardLink() }
                if let url = session.currentURL {
                    Button("Copier le lien", systemImage: "link") { UIPasteboard.general.url = url }
                }
                Divider()
                Button("Réglages", systemImage: "gearshape") { onSettings() }
            } label: {
                Image(systemName: "ellipsis.circle")
                    .font(.title3)
                    .frame(width: 32, height: 32)
            }
            .accessibilityLabel("Menu")
        }
        .tint(tint)
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(tint.opacity(0.10))
        .overlay(alignment: .bottom) {
            Rectangle().fill(tint).frame(height: 2)
        }
        .animation(.easeOut(duration: 0.15), value: selected.id)
    }
}

struct AccountChip: View {
    let account: Account
    let isSelected: Bool
    var action: () -> Void

    var body: some View {
        let tint = account.color.color
        Button(action: action) {
            HStack(spacing: 6) {
                Circle().fill(tint).frame(width: 8, height: 8)
                Text(account.name)
                    .font(.subheadline.weight(isSelected ? .semibold : .regular))
                    .lineLimit(1)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .background(
                Capsule().fill(isSelected ? tint.opacity(0.22) : Color(.secondarySystemBackground))
            )
            .overlay(
                Capsule().strokeBorder(isSelected ? tint : .clear, lineWidth: 1.5)
            )
            .foregroundStyle(.primary)
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(isSelected ? .isSelected : [])
    }
}

struct AccountPane: View {
    @ObservedObject var session: WebSession
    let account: Account

    var body: some View {
        WebViewContainer(webView: session.webView)
            .overlay(alignment: .top) {
                if session.isLoading {
                    ProgressView(value: session.progress)
                        .progressViewStyle(.linear)
                        .tint(account.color.color)
                }
            }
            .sheet(item: $session.popup) { popup in
                PopupView(popup: popup, tint: account.color.color) {
                    session.popup = nil
                }
            }
    }
}

struct PopupView: View {
    @ObservedObject var popup: PopupSession
    let tint: Color
    var onDismiss: () -> Void

    var body: some View {
        NavigationStack {
            WebViewContainer(webView: popup.webView)
                .ignoresSafeArea(.container, edges: .bottom)
                .navigationTitle(popup.displayTitle)
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button("Fermer", action: onDismiss)
                    }
                    ToolbarItem(placement: .primaryAction) {
                        if let url = popup.url {
                            Button {
                                UIApplication.shared.open(url)
                            } label: {
                                Image(systemName: "safari")
                            }
                            .accessibilityLabel("Ouvrir dans Safari")
                        }
                    }
                }
        }
        .tint(tint)
    }
}

/// Affiche une WKWebView existante (créée et conservée par sa session).
struct WebViewContainer: UIViewRepresentable {
    let webView: WKWebView

    func makeUIView(context: Context) -> WKWebView { webView }
    func updateUIView(_ uiView: WKWebView, context: Context) {}
}
