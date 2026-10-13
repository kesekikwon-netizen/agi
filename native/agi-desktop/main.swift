import Cocoa
import WebKit

class AppDelegate: NSObject, NSApplicationDelegate, WKNavigationDelegate {
    var window: NSWindow!
    var webView: WKWebView!

    func applicationDidFinishLaunching(_ notification: Notification) {
        let screenRect = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        let winWidth: CGFloat = min(1520, screenRect.width * 0.95)
        let winHeight: CGFloat = min(980, screenRect.height * 0.95)
        let winRect = NSRect(
            x: (screenRect.width - winWidth) / 2 + screenRect.minX,
            y: (screenRect.height - winHeight) / 2 + screenRect.minY,
            width: winWidth,
            height: winHeight
        )

        window = NSWindow(
            contentRect: winRect,
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "AGI — Archaeological Geometric Intelligence (고고학 3D 사진측량 & CAD 도면화)"
        window.center()

        let config = WKWebViewConfiguration()
        config.preferences.setValue(true, forKey: "allowFileAccessFromFileURLs")
        
        webView = WKWebView(frame: window.contentView!.bounds, configuration: config)
        webView.autoresizingMask = [.width, .height]
        webView.navigationDelegate = self
        window.contentView?.addSubview(webView)

        let filePath = "/Users/kwon/Documents/agi/docs/blueprint/index.html"
        let fileUrl = URL(fileURLWithPath: filePath)
        webView.loadFileURL(fileUrl, allowingReadAccessTo: URL(fileURLWithPath: "/Users/kwon/Documents/agi/docs/"))

        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return true
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.regular)
let delegate = AppDelegate()
app.delegate = delegate
app.run()
