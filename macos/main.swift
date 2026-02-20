import Cocoa
import InputMethodKit

// The main entry point for the macOS InputMethodKit server.
// Runs the application loop just like the old Rust code did.

let connectionName = "com.manhpham.inputmethod.xkey"
let bundleId = "com.manhpham.inputmethod.xkey"

guard let server = IMKServer(name: connectionName, bundleIdentifier: bundleId) else {
    fatalError("Failed to create IMKServer")
}

print("XKey macOS InputMethod running natively in Swift.")

// Start the NSApplication run loop
NSApplication.shared.run()
