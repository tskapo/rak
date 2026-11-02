import SwiftUI

@main
struct DuoApp: App {
    @StateObject private var accounts = AccountStore()
    @StateObject private var sessions = SessionManager()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(accounts)
                .environmentObject(sessions)
        }
    }
}
