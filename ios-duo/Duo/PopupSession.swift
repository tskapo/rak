import SwiftUI
import UIKit
import WebKit

/// Fenêtre secondaire (lien externe, OAuth…). Elle partage l'espace de données
/// du compte qui l'a ouverte.
@MainActor
final class PopupSession: NSObject, ObservableObject, Identifiable {
    let id = UUID()
    let webView: WKWebView

    @Published private(set) var title = ""
    @Published private(set) var url: URL?

    var onClose: (() -> Void)?

    private var observations: [NSKeyValueObservation] = []

    init(configuration: WKWebViewConfiguration) {
        webView = WKWebView(frame: .zero, configuration: configuration)
        super.init()
        webView.navigationDelegate = self
        webView.uiDelegate = self
        webView.allowsBackForwardNavigationGestures = true

        observations = [
            webView.observe(\.title, options: [.initial, .new]) { [weak self] _, _ in
                Task { @MainActor in self?.title = self?.webView.title ?? "" }
            },
            webView.observe(\.url, options: [.initial, .new]) { [weak self] _, _ in
                Task { @MainActor in self?.url = self?.webView.url }
            },
        ]
    }

    var displayTitle: String {
        if !title.isEmpty { return title }
        return url?.host ?? ""
    }
}

extension PopupSession: WKNavigationDelegate, WKUIDelegate {
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction) async -> WKNavigationActionPolicy {
        if let url = navigationAction.request.url, !ClaudeHosts.isWebScheme(url) {
            UIApplication.shared.open(url, options: [:], completionHandler: nil)
            return .cancel
        }
        return .allow
    }

    // window.close() appelé par la page (fin d'un flux OAuth par exemple).
    func webViewDidClose(_ webView: WKWebView) {
        onClose?()
    }

    // Une popup qui ouvre une autre fenêtre : on charge simplement dans celle-ci.
    func webView(_ webView: WKWebView,
                 createWebViewWith configuration: WKWebViewConfiguration,
                 for navigationAction: WKNavigationAction,
                 windowFeatures: WKWindowFeatures) -> WKWebView? {
        webView.load(navigationAction.request)
        return nil
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

/// alert() / confirm() JavaScript affichés avec des alertes iOS.
@MainActor
enum JSDialogs {
    static func alert(_ message: String, host: String) async {
        await withCheckedContinuation { (continuation: CheckedContinuation<Void, Never>) in
            guard let presenter = topViewController() else {
                continuation.resume()
                return
            }
            let alert = UIAlertController(title: host, message: message, preferredStyle: .alert)
            alert.addAction(UIAlertAction(title: "OK", style: .default) { _ in continuation.resume() })
            presenter.present(alert, animated: true)
        }
    }

    static func confirm(_ message: String, host: String) async -> Bool {
        await withCheckedContinuation { (continuation: CheckedContinuation<Bool, Never>) in
            guard let presenter = topViewController() else {
                continuation.resume(returning: false)
                return
            }
            let alert = UIAlertController(title: host, message: message, preferredStyle: .alert)
            alert.addAction(UIAlertAction(title: "Annuler", style: .cancel) { _ in continuation.resume(returning: false) })
            alert.addAction(UIAlertAction(title: "OK", style: .default) { _ in continuation.resume(returning: true) })
            presenter.present(alert, animated: true)
        }
    }

    private static func topViewController() -> UIViewController? {
        let scenes = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }
        let scene = scenes.first { $0.activationState == .foregroundActive } ?? scenes.first
        var controller = scene?.keyWindow?.rootViewController
        while let presented = controller?.presentedViewController {
            controller = presented
        }
        return controller
    }
}
