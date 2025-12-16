import SwiftUI
import L3dKit

@main
struct L3dViewerApp: App {
    #if os(macOS)
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    #endif

    var body: some Scene {
        WindowGroup("L3D Viewer", id: "viewer") {
            ContentView()
        }
        #if os(macOS)
        .windowToolbarStyle(.unified(showsTitle: true))
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Open L3D File...") {
                    NotificationCenter.default.post(name: .openFile, object: nil)
                }
                .keyboardShortcut("o", modifiers: .command)
            }
        }
        #endif
    }
}

extension Notification.Name {
    static let openFile = Notification.Name("openFile")
    static let openExternalFile = Notification.Name("openExternalFile")
}

#if os(macOS)
class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return true
    }

    func application(_ application: NSApplication, open urls: [URL]) {
        for url in urls {
            if url.pathExtension.lowercased() == "l3d" {
                NotificationCenter.default.post(name: .openExternalFile, object: url)
                break
            }
        }
    }
}
#endif
