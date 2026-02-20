import Cocoa
import InputMethodKit

@objc(XKeyInputController)
class XKeyInputController: IMKInputController {
    
    // Pointer to the Rust CoreState instance
    private var coreState: UnsafeMutableRawPointer?
    
    // Keep track of whether we've just committed text to handle HidePreedit correctly
    private var justCommitted = false
    
    override init!(server: IMKServer!, delegate: Any!, client inputClient: Any!) {
        super.init(server: server, delegate: delegate, client: inputClient)
        // Initialize the Rust component
        self.coreState = xkey_core_create()
    }
    
    deinit {
        if let state = self.coreState {
            xkey_core_destroy(state)
        }
    }
    
    // MARK: - Callbacks for Rust C-API
    
    private func getCallbacks() -> XKeyCallbacks {
        let context = Unmanaged.passUnretained(self).toOpaque()
        
        // update_preedit callback
        let updatePreeditObjc: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<UInt8>?, Int, Int, Bool) -> Void = { ctx, textPtr, textLen, caret, visible in
            guard let ctx = ctx else { return }
            let controller = Unmanaged<XKeyInputController>.fromOpaque(ctx).takeUnretainedValue()
            if visible {
                if let ptr = textPtr {
                    let data = Data(bytes: ptr, count: textLen)
                    if let text = String(data: data, encoding: .utf8) {
                        controller.dispatchUpdatePreedit(text: text, caret: caret)
                    }
                }
            }
            controller.justCommitted = false
        }
        
        // hide_preedit callback
        let hidePreeditObjc: @convention(c) (UnsafeMutableRawPointer?) -> Void = { ctx in
            guard let ctx = ctx else { return }
            let controller = Unmanaged<XKeyInputController>.fromOpaque(ctx).takeUnretainedValue()
            if !controller.justCommitted {
                controller.dispatchHidePreedit()
            }
            controller.justCommitted = false
        }
        
        // commit callback
        let commitObjc: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<UInt8>?, Int) -> Void = { ctx, textPtr, textLen in
            guard let ctx = ctx else { return }
            let controller = Unmanaged<XKeyInputController>.fromOpaque(ctx).takeUnretainedValue()
            if let ptr = textPtr {
                let data = Data(bytes: ptr, count: textLen)
                if let text = String(data: data, encoding: .utf8) {
                    controller.dispatchCommit(text: text)
                }
            }
            controller.justCommitted = true
        }

        return XKeyCallbacks(
            context: context,
            update_preedit: updatePreeditObjc,
            hide_preedit: hidePreeditObjc,
            commit: commitObjc
        )
    }
    
    // MARK: - Dispatch Methods for IMK operations
    
    private func dispatchUpdatePreedit(text: String, caret: Int) {
        guard let client = self.client() else { return }
        
        let nsText = NSAttributedString(string: text)
        let selectionRange = NSMakeRange(caret, 0)
        let replacementRange = NSMakeRange(NSNotFound, NSNotFound)
        
        client.setMarkedText(nsText, selectionRange: selectionRange, replacementRange: replacementRange)
    }
    
    private func dispatchHidePreedit() {
        guard let client = self.client() else { return }
        
        let emptyText = NSAttributedString(string: "")
        let selectionRange = NSMakeRange(0, 0)
        let replacementRange = NSMakeRange(NSNotFound, NSNotFound)
        
        client.setMarkedText(emptyText, selectionRange: selectionRange, replacementRange: replacementRange)
    }
    
    private func dispatchCommit(text: String) {
        guard let client = self.client() else { return }
        let replacementRange = NSMakeRange(NSNotFound, NSNotFound)
        client.insertText(text, replacementRange: replacementRange)
    }
    
    // MARK: - InputMethodKit overrides
    
    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event = event, let state = coreState else {
            return false
        }
        
        // Only process key down events
        if event.type != .keyDown {
            return false
        }
        
        // Convert macOS modifier flags to X11 state format
        var modifierState: UInt32 = 0
        let flags = event.modifierFlags
        if flags.contains(.control) {
            modifierState |= (1 << 2) // CTRL
        }
        if flags.contains(.option) {
            modifierState |= (1 << 3) // ALT
        }
        if flags.contains(.command) {
            modifierState |= (1 << 26) // SUPER/CMD
        }
        
        // Convert key code to X11 keysym
        let keyCode = event.keyCode
        var keyval: UInt32? = nil
        
        switch keyCode {
        case 51: keyval = 0xff08 // Backspace
        case 36: keyval = 0xff0d // Return
        case 48: keyval = 0xff09 // Tab
        case 53: keyval = 0xff1b // Escape
        case 49: keyval = 0x20   // Space
        default:
            if let chars = event.characters, let firstChar = chars.first {
                if firstChar.isASCII && (firstChar.isLetter || firstChar.isNumber || firstChar.isPunctuation || firstChar.isWhitespace) {
                    if let asciiValue = firstChar.asciiValue {
                        keyval = UInt32(asciiValue)
                    }
                }
            }
        }
        
        guard let kv = keyval else {
            return false
        }
        
        // Invoke Rust core to handle key
        let callbacks = getCallbacks()
        let consumed = xkey_core_handle_key(state, kv, modifierState, callbacks)
        return consumed
    }
    
    override func activateServer(_ sender: Any!) {
        // Reset buffer on activate
        if let state = coreState {
            xkey_core_clear_buffer(state)
        }
    }
    
    override func deactivateServer(_ sender: Any!) {
        // Flush buffer on deactivate
        if let state = coreState {
            let callbacks = getCallbacks()
            xkey_core_flush_buffer(state, callbacks)
        }
    }
}
