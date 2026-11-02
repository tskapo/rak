import SwiftUI
import UIKit
import WebKit

enum Destination {
    case chat, code

    var url: URL {
        switch self {
        case .chat: return URL(string: "https://claude.ai/new")!
        case .code: return URL(string: "https://claude.ai/code")!
        }
    }
}

enum ClaudeHosts {
    private static let domains = ["claude.ai", "claude.com", "anthropic.com"]

    static func contains(_ url: URL?) -> Bool {
        guard let host = url?.host?.lowercased() else { return false }
        return domains.contains { host == $0 || host.hasSuffix("." + $0) }
    }

    /// Schémas que la WebView sait afficher ; le reste (mailto:, tel:, …) part vers iOS.
    static func isWebScheme(_ url: URL) -> Bool {
        ["http", "https", "about", "blob", "data"].contains(url.scheme?.lowercased() ?? "")
    }
}

/// Une WebView par compte, avec son propre stockage (cookies, localStorage,
/// IndexedDB, cache). Deux sessions ne partagent jamais rien.
@MainActor
final class WebSession: NSObject, ObservableObject {
    let accountID: UUID
    let webView: WKWebView

    @Published private(set) var canGoBack = false
    @Published private(set) var canGoForward = false
    @Published private(set) var isLoading = false
    @Published private(set) var progress: Double = 0
    /// Fenêtre ouverte par la page (lien externe, connexion OAuth GitHub/Google Drive…).
    @Published var popup: PopupSession?

    private var observations: [NSKeyValueObservation] = []

    init(accountID: UUID) {
        self.accountID = accountID
        self.webView = WKWebView(frame: .zero, configuration: Self.makeConfiguration(for: accountID))
        super.init()

        webView.navigationDelegate = self
        webView.uiDelegate = self
        webView.allowsBackForwardNavigationGestures = true
        #if DEBUG
        webView.isInspectable = true
        #endif

        observeState()
        open(.chat)
    }

    static func makeConfiguration(for accountID: UUID) -> WKWebViewConfiguration {
        let config = WKWebViewConfiguration()
        // Le cœur de l'app : un espace de données persistant et isolé par compte (iOS 17+).
        config.websiteDataStore = WKWebsiteDataStore(forIdentifier: accountID)
        config.allowsInlineMediaPlayback = true
        config.mediaTypesRequiringUserActionForPlayback = []
        config.defaultWebpagePreferences.preferredContentMode = .mobile
        // Se présenter comme Safari mobile pour que le site serve sa version normale.
        config.applicationNameForUserAgent = "Version/17.0 Mobile/15E148 Safari/604.1"
        return config
    }

    private func observeState() {
        observations = [
            webView.observe(\.canGoBack, options: [.initial, .new]) { [weak self] _, _ in
                Task { @MainActor in self?.syncState() }
            },
            webView.observe(\.canGoForward, options: [.initial, .new]) { [weak self] _, _ in
                Task { @MainActor in self?.syncState() }
            },
            webView.observe(\.isLoading, options: [.initial, .new]) { [weak self] _, _ in
                Task { @MainActor in self?.syncState() }
            },
            webView.observe(\.estimatedProgress, options: [.initial, .new]) { [weak self] _, _ in
                Task { @MainActor in self?.syncState() }
            },
        ]
    }

    private func syncState() {
        canGoBack = webView.canGoBack
        canGoForward = webView.canGoForward
        isLoading = webView.isLoading
        progress = webView.estimatedProgress
    }

    // MARK: Actions

    func open(_ destination: Destination) {
        webView.load(URLRequest(url: destination.url))
    }

    func goBack() { webView.goBack() }
    func goForward() { webView.goForward() }
    func reload() { webView.reload() }

    var currentURL: URL? { webView.url }

    /// Ouvre dans ce compte un lien claude.ai copié (ex. lien de connexion reçu par e-mail).
    func openPasteboardLink() {
        let pasteboard = UIPasteboard.general
        let candidate = pasteboard.url ?? pasteboard.string
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .flatMap(URL.init(string:))
        guard let url = candidate, ClaudeHosts.contains(url), url.scheme?.lowercased() == "https" else { return }
        webView.load(URLRequest(url: url))
    }

    /// Déconnecte le compte : efface toutes les données de son espace isolé.
    func clearData() async {
        popup = nil
        let store = webView.configuration.websiteDataStore
        await store.removeData(ofTypes: WKWebsiteDataStore.allWebsiteDataTypes(), modifiedSince: .distantPast)
        open(.chat)
    }

    private func presentPopup(configuration: WKWebViewConfiguration, loading url: URL? = nil) -> PopupSession {
        let popup = PopupSession(configuration: configuration)
        popup.onClose = { [weak self] in self?.popup = nil }
        if let url { popup.webView.load(URLRequest(url: url)) }
        self.popup = popup
        return popup
    }
}

extension WebSession: WKNavigationDelegate, WKUIDelegate {
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction) async -> WKNavigationActionPolicy {
        guard let url = navigationAction.request.url else { return .allow }

        if !ClaudeHosts.isWebScheme(url) {
            UIApplication.shared.open(url, options: [:], completionHandler: nil)
            return .cancel
        }

        // Lien externe touché dans une réponse : on l'ouvre dans une feuille par-dessus,
        // avec le même espace de données (donc le même compte), sans quitter la conversation.
        if navigationAction.navigationType == .linkActivated,
           navigationAction.targetFrame?.isMainFrame == true,
           !ClaudeHosts.contains(url) {
            _ = presentPopup(configuration: Self.makeConfiguration(for: accountID), loading: url)
            return .cancel
        }

        return .allow
    }

    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
        webView.reload()
    }

    // window.open / target="_blank"
    func webView(_ webView: WKWebView,
                 createWebViewWith configuration: WKWebViewConfiguration,
                 for navigationAction: WKNavigationAction,
                 windowFeatures: WKWindowFeatures) -> WKWebView? {
        if let url = navigationAction.request.url, ClaudeHosts.contains(url) {
            webView.load(navigationAction.request)
            return nil
        }
        // La WebView renvoyée doit utiliser `configuration` : elle partage ainsi le stockage
        // du compte et garde window.opener (nécessaire aux connexions OAuth).
        return presentPopup(configuration: configuration).webView
    }

    func webView(_ webView: WKWebView,
                 requestMediaCapturePermissionFor origin: WKSecurityOrigin,
                 initiatedByFrame frame: WKFrameInfo,
                 type: WKMediaCaptureType) async -> WKPermissionDecision {
        ClaudeHosts.contains(URL(string: "https://\(origin.host)")) ? .grant : .prompt
    }

    func webView(_ webView: WKWebView,
                 runJavaScriptAlertPanelWithMessage message: String,
                 initiatedByFrame frame: WKFrameInfo) async {
        await JSDialogs.alert(message, host: frame.securityOrigin.host)
    }

    func webView(_ webView: WKWebView,
                 runJavaScriptConfirmPanelWithMessage message: String,
                 initiatedByFrame frame: WKFrameInfo) async -> Bool {
        await JSDialogs.confirm(message, host: frame.securityOrigin.host)
    }
}

/// Session gérée par compte, conservée en mémoire pour que chaque compte reste
/// chargé quand on passe de l'un à l'autre.
@MainActor
final class SessionManager: ObservableObject {
    private var sessions: [UUID: WebSession] = [:]

    func session(for accountID: UUID) -> WebSession {
        if let existing = sessions[accountID] { return existing }
        let session = WebSession(accountID: accountID)
        sessions[accountID] = session
        return session
    }

    /// Supprime définitivement les données d'un compte retiré de l'app.
    func discard(_ accountID: UUID) async {
        if let session = sessions.removeValue(forKey: accountID) {
            await session.clearData()
        }
        try? await WKWebsiteDataStore.remove(forIdentifier: accountID)
    }
}
