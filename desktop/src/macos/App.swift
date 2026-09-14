import AppKit
import UniformTypeIdentifiers

typealias Bridge = @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> UnsafePointer<CChar>?

final class SessionRow: NSTableRowView {
    var selectionColor = NSColor.selectedControlColor
    override func drawSelection(in dirtyRect: NSRect) { selectionColor.setFill(); bounds.fill() }
    override var interiorBackgroundStyle: NSView.BackgroundStyle { .normal }
}

// AppKit controls keep keyboard navigation, text selection and VoiceOver native.
final class WaidApp: NSObject, NSApplicationDelegate, NSWindowDelegate, NSTableViewDataSource, NSTableViewDelegate, NSSearchFieldDelegate {
    let context: UnsafeMutableRawPointer?
    let bridge: Bridge
    let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 660, height: 800),
        styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
    let search = NSSearchField()
    let status = NSPopUpButton()
    let agent = NSPopUpButton()
    let language = NSPopUpButton()
    let table = NSTableView()
    let split = NSSplitView()
    let detail = NSTextView()
    let notice = NSTextField(wrappingLabelWithString: "")
    let opacityLabel = NSTextField(labelWithString: "")
    let opacity = NSSlider(value: 100, minValue: 40, maxValue: 100, target: nil, action: nil)
    let editor = NSTextView()
    var editorWindow: NSWindow?
    var statusItem: NSStatusItem!
    var timer: Timer?
    var view: [String: Any] = [:]
    var rows: [[String: Any]] = []
    var buttons: [String: NSButton] = [:]
    var rendering = false
    var lastRows = ""
    var lastSkin = ""
    var lastLanguage = ""
    var skin: [String: Any] = [:]
    var smokeAttempts = 0

    init(_ context: UnsafeMutableRawPointer?, _ bridge: @escaping Bridge) {
        self.context = context; self.bridge = bridge
        super.init()
    }

    func request(_ action: String, value: String = "", id: String? = nil) {
        var message = ["action": action, "value": value]
        if let id = id ?? selected?["id"] as? String { message["id"] = id }
        let data = try! JSONSerialization.data(withJSONObject: message)
        let text = String(decoding: data, as: UTF8.self)
        view = text.withCString { ptr in
            guard let response = bridge(context, ptr), let data = String(cString: response).data(using: .utf8),
                let result = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return [:] }
            return result
        }
        render()
    }

    var selected: [String: Any]? { rows.indices.contains(table.selectedRow) ? rows[table.selectedRow] : nil }

    func tr(_ ko: String, _ en: String) -> String { view["language"] as? String == "ko" ? ko : en }

    func localize() {
        search.placeholderString = tr("작업, 프로젝트, 모델, 폴더 검색", "Search tasks, projects, models and folders")
        search.setAccessibilityLabel(tr("세션 검색", "Search sessions"))
        status.removeAllItems()
        status.addItems(withTitles: [tr("모든 상태", "All statuses"), tr("내 차례", "Waiting"), tr("작업 중", "Working"),
            tr("오류", "Error"), tr("유휴", "Idle"), tr("목록 제외", "Dismissed"), tr("미확인", "Unknown")])
        status.setAccessibilityLabel(tr("상태 필터", "Status filter"))
        agent.setAccessibilityLabel(tr("에이전트 필터", "Agent filter"))
        language.setAccessibilityLabel(tr("언어", "Language"))
        opacityLabel.stringValue = tr("불투명도", "Opacity")
        opacity.setAccessibilityLabel(tr("창 불투명도, 40~100퍼센트", "Window opacity, 40 to 100 percent"))
        table.tableColumns.first?.title = tr("세션", "Sessions")
        table.setAccessibilityLabel(tr("코딩 에이전트 세션", "Coding agent sessions"))
        detail.setAccessibilityLabel(tr("선택한 세션 상세", "Selected session details"))
        editor.setAccessibilityLabel(tr("템플릿 JSON 편집기", "Template JSON editor"))
        editorWindow?.title = tr("템플릿 — 복제, 편집, 미리보기, 적용", "Templates — duplicate, edit, preview, apply")
        statusItem.button?.setAccessibilityLabel(tr("waid 세션 관리자", "waid session manager"))
        for (action, ko, en) in [("open", "세션 열기", "Open session"), ("copy", "요청 복사", "Copy prompt"),
            ("connect", "복사한 링크 연결", "Connect copied link"), ("disconnect", "연결 해제", "Clear connection"),
            ("editor", "템플릿…", "Templates…"), ("day", "밝은 테마", "Daylight"), ("night", "어두운 테마", "Midnight"),
            ("import", "가져오기…", "Import…"), ("export", "내보내기…", "Export…"),
            ("preview", "미리보기", "Preview"), ("template", "적용", "Apply")] {
            buttons[action]?.title = tr(ko, en)
        }
        buildMenus()
    }

    func button(_ title: String, _ action: String, key: String = "") -> NSButton {
        let button = NSButton(title: title, target: self, action: #selector(clicked(_:)))
        button.identifier = NSUserInterfaceItemIdentifier(action)
        if !key.isEmpty { button.keyEquivalent = key; button.keyEquivalentModifierMask = .command }
        buttons[action] = button
        return button
    }

    func stack(_ views: [NSView], vertical: Bool = false) -> NSStackView {
        let result = NSStackView(views: views)
        result.orientation = vertical ? .vertical : .horizontal
        result.distribution = .fill
        result.alignment = vertical ? .leading : .centerY
        result.spacing = 8
        return result
    }

    func scroll(_ document: NSView) -> NSScrollView {
        let scroll = NSScrollView()
        scroll.hasVerticalScroller = true
        scroll.borderType = .bezelBorder
        scroll.documentView = document
        if let text = document as? NSTextView {
            text.isVerticallyResizable = true; text.isHorizontallyResizable = false
            text.autoresizingMask = [.width]
            text.textContainer?.widthTracksTextView = true
            text.textContainerInset = NSSize(width: 10, height: 10)
        }
        return scroll
    }

    func menuItem(_ title: String, _ action: Selector, key: String = "") -> NSMenuItem {
        let item = NSMenuItem(title: title, action: action, keyEquivalent: key)
        item.target = self
        return item
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        window.title = "waid — what am I doing?"
        window.minSize = NSSize(width: 640, height: 650)
        window.isReleasedWhenClosed = false
        window.delegate = self
        window.setFrameAutosaveName("waid.main")
        search.delegate = self
        status.target = self; status.action = #selector(filterChanged(_:))
        agent.target = self; agent.action = #selector(filterChanged(_:))
        language.addItems(withTitles: ["한국어", "English"])
        language.target = self; language.action = #selector(languageChanged)
        opacity.target = self; opacity.action = #selector(opacityChanged)
        opacity.isContinuous = false
        opacity.widthAnchor.constraint(equalToConstant: 100).isActive = true

        let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("session"))
        column.resizingMask = .autoresizingMask
        table.addTableColumn(column); table.headerView = nil
        table.columnAutoresizingStyle = .lastColumnOnlyAutoresizingStyle
        table.dataSource = self; table.delegate = self
        table.target = self; table.doubleAction = #selector(openSession)
        table.allowsEmptySelection = true
        detail.isEditable = false; detail.isSelectable = true
        let listScroll = scroll(table)
        let detailScroll = scroll(detail)
        listScroll.heightAnchor.constraint(greaterThanOrEqualToConstant: 180).isActive = true
        detailScroll.heightAnchor.constraint(greaterThanOrEqualToConstant: 150).isActive = true
        split.isVertical = false; split.dividerStyle = .thin
        split.setContentHuggingPriority(.defaultLow, for: .vertical)
        split.addArrangedSubview(listScroll); split.addArrangedSubview(detailScroll)
        split.setHoldingPriority(.defaultLow, forSubviewAt: 0)
        split.setHoldingPriority(.defaultLow, forSubviewAt: 1)
        let filters = stack([status, agent, button("Show all", "all"), button("Include auxiliary", "aux")])
        let actions = stack([button("Open session", "open", key: "\r"), button("Pin", "pin", key: "p"),
            button("Hide", "hide"), button("Do not show in waid", "dismiss", key: "d")])
        let connections = stack([button("Copy prompt", "copy"), button("Connect copied link", "connect"), button("Clear connection", "disconnect")])
        let preferences = stack([button("Always on top", "top"), opacityLabel, opacity, button("Templates…", "editor"), language])
        let root = stack([search, filters, actions, connections, split, notice, preferences], vertical: true)
        for control in [search, filters, actions, connections, preferences] as [NSView] {
            control.heightAnchor.constraint(equalToConstant: 26).isActive = true
        }
        root.translatesAutoresizingMaskIntoConstraints = false
        window.contentView!.addSubview(root)
        NSLayoutConstraint.activate([
            root.leadingAnchor.constraint(equalTo: window.contentView!.leadingAnchor, constant: 12),
            root.trailingAnchor.constraint(equalTo: window.contentView!.trailingAnchor, constant: -12),
            root.topAnchor.constraint(equalTo: window.contentView!.topAnchor, constant: 12),
            root.bottomAnchor.constraint(equalTo: window.contentView!.bottomAnchor, constant: -12),
            search.widthAnchor.constraint(equalTo: root.widthAnchor),
            split.widthAnchor.constraint(equalTo: root.widthAnchor),
            notice.widthAnchor.constraint(equalTo: root.widthAnchor)
        ])
        notice.maximumNumberOfLines = 4
        notice.setContentHuggingPriority(.required, for: .vertical)
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        statusItem.button?.title = "waid"
        request("poll")
        showWindow()
        timer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { [weak self] _ in self?.request("poll") }
        if CommandLine.arguments.contains("--macos-smoke") {
            DispatchQueue.main.async { self.smoke() }
        }
    }

    func buildMenus() {
        let main = NSMenu()
        let appMenu = NSMenu()
        appMenu.addItem(menuItem(tr("waid 열기", "Show waid"), #selector(showWindow)))
        appMenu.addItem(menuItem(tr("폰트 라이선스", "Font license"), #selector(showLicense)))
        appMenu.addItem(.separator())
        appMenu.addItem(menuItem(tr("waid 숨기기", "Hide waid"), #selector(hideApp), key: "h"))
        appMenu.addItem(menuItem(tr("waid 종료", "Quit waid"), #selector(quit), key: "q"))
        let appItem = NSMenuItem(); appItem.submenu = appMenu; main.addItem(appItem)
        let edit = NSMenu(title: tr("편집", "Edit"))
        for (title, selector, key) in [(tr("실행 취소", "Undo"), Selector(("undo:")), "z"), (tr("잘라내기", "Cut"), #selector(NSText.cut(_:)), "x"),
            (tr("복사", "Copy"), #selector(NSText.copy(_:)), "c"), (tr("붙여넣기", "Paste"), #selector(NSText.paste(_:)), "v"), (tr("전체 선택", "Select All"), #selector(NSText.selectAll(_:)), "a")] {
            edit.addItem(NSMenuItem(title: title, action: selector, keyEquivalent: key))
        }
        edit.addItem(menuItem(tr("세션 찾기", "Find sessions"), #selector(find), key: "f"))
        let editItem = NSMenuItem(); editItem.submenu = edit; main.addItem(editItem)
        let windows = NSMenu(title: tr("창", "Window"))
        windows.addItem(NSMenuItem(title: tr("최소화", "Minimize"), action: #selector(NSWindow.performMiniaturize(_:)), keyEquivalent: "m"))
        windows.addItem(NSMenuItem(title: tr("닫기", "Close"), action: #selector(NSWindow.performClose(_:)), keyEquivalent: "w"))
        let windowItem = NSMenuItem(); windowItem.submenu = windows; main.addItem(windowItem)
        NSApp.mainMenu = main; NSApp.windowsMenu = windows
        let tray = NSMenu()
        tray.addItem(menuItem(tr("waid 열기", "Show waid"), #selector(showWindow)))
        tray.addItem(menuItem(tr("waid 종료", "Quit waid"), #selector(quit)))
        statusItem.menu = tray
    }

    func color(_ text: String) -> NSColor {
        let n = UInt32(text.dropFirst(), radix: 16) ?? 0
        return NSColor(srgbRed: CGFloat((n >> 16) & 255) / 255, green: CGFloat((n >> 8) & 255) / 255,
            blue: CGFloat(n & 255) / 255, alpha: 1)
    }

    func render() {
        rendering = true; defer { rendering = false }
        let id = selected?["id"] as? String
        let nextLanguage = view["language"] as? String ?? "en"
        if nextLanguage != lastLanguage { lastLanguage = nextLanguage; localize() }
        language.selectItem(at: nextLanguage == "ko" ? 0 : 1)
        if search.currentEditor() == nil { search.stringValue = view["search"] as? String ?? "" }
        let states = ["", "waiting", "working", "error", "idle", "done", "unknown"]
        status.selectItem(at: states.firstIndex(of: view["state"] as? String ?? "") ?? 0)
        let agents = [tr("모든 에이전트", "All agents")] + (view["agents"] as? [String] ?? [])
        if agent.itemTitles != agents { agent.removeAllItems(); agent.addItems(withTitles: agents) }
        agent.selectItem(at: agents.firstIndex(of: view["agent"] as? String ?? "") ?? 0)
        for (key, title) in [("all", tr("전체 보기", "Show all")), ("aux", tr("보조 세션 포함", "Include auxiliary")), ("top", tr("항상 위", "Always on top"))] {
            buttons[key]?.title = ((view[key] as? Bool ?? false) ? "✓ " : "") + title
        }
        window.level = view["top"] as? Bool == true ? .floating : .normal
        window.alphaValue = CGFloat(view["opacity"] as? Int ?? 100) / 100
        opacity.integerValue = view["opacity"] as? Int ?? 100
        skin = view["skin"] as? [String: Any] ?? [:]
        let colors = skin["colors"] as? [String: String] ?? [:]
        window.backgroundColor = color(colors["background"] ?? "#FFFFFF")
        window.contentView!.wantsLayer = true
        window.contentView!.layer!.backgroundColor = window.backgroundColor.cgColor
        let background = window.backgroundColor.usingColorSpace(.sRGB)!
        let brightness = (background.redComponent + background.greenComponent + background.blueComponent) / 3
        window.appearance = NSAppearance(named: brightness < 0.5 ? .darkAqua : .aqua)
        table.backgroundColor = color(colors["surface"] ?? "#FFFFFF")
        detail.backgroundColor = table.backgroundColor
        detail.textColor = color(colors["text"] ?? "#101B38")
        detail.font = .systemFont(ofSize: CGFloat(skin["font_size"] as? Int ?? 14))
        notice.textColor = color(colors["muted"] ?? "#3F4F6D")
        let messages = [view["error"] as? String ?? "", view["notice"] as? String ?? ""] + (view["warnings"] as? [String] ?? [])
        notice.stringValue = messages.filter { !$0.isEmpty }.joined(separator: "\n")
        let nextRows = view["rows"] as? [[String: Any]] ?? []
        let signature = String(decoding: try! JSONSerialization.data(withJSONObject: nextRows, options: [.sortedKeys]), as: UTF8.self)
        let style = String(decoding: try! JSONSerialization.data(withJSONObject: skin, options: [.sortedKeys]), as: UTF8.self)
        if signature != lastRows || style != lastSkin {
            lastRows = signature; lastSkin = style; rows = nextRows
            let origin = table.enclosingScrollView?.contentView.bounds.origin ?? .zero
            table.reloadData()
            if let index = rows.firstIndex(where: { $0["id"] as? String == id }) {
                table.selectRowIndexes(IndexSet(integer: index), byExtendingSelection: false)
            } else { table.deselectAll(nil) }
            table.enclosingScrollView?.contentView.scroll(to: origin)
        }
        updateSelection()
    }

    func numberOfRows(in tableView: NSTableView) -> Int { rows.count }
    func tableView(_ tableView: NSTableView, heightOfRow row: Int) -> CGFloat {
        let count = (rows[row]["lines"] as? [String] ?? []).count
        let font = skin["font_size"] as? Int ?? 14
        let gap = skin["line_gap"] as? Int ?? 2
        let padding = skin["padding"] as? Int ?? 10
        return CGFloat(count * (font + gap + 4) + 2 * padding)
    }
    func tableView(_ tableView: NSTableView, rowViewForRow row: Int) -> NSTableRowView? {
        let view = SessionRow()
        view.selectionColor = color((skin["colors"] as? [String: String])?["selection"] ?? "#EDF3FF")
        return view
    }
    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        let item = rows[row]
        let text = NSTextField(wrappingLabelWithString: (item["lines"] as? [String] ?? []).joined(separator: "\n"))
        text.maximumNumberOfLines = (item["lines"] as? [String] ?? []).count
        text.lineBreakMode = .byTruncatingTail
        text.font = .systemFont(ofSize: CGFloat(skin["font_size"] as? Int ?? 14))
        let colors = skin["colors"] as? [String: String] ?? [:]
        text.textColor = color(colors["text"] ?? "#101B38")
        let styled = NSMutableAttributedString(string: text.stringValue)
        let paragraph = NSMutableParagraphStyle()
        paragraph.lineSpacing = CGFloat(skin["line_gap"] as? Int ?? 2)
        paragraph.lineBreakMode = .byTruncatingTail
        styled.addAttributes([.font: text.font!, .foregroundColor: text.textColor!, .paragraphStyle: paragraph], range: NSRange(location: 0, length: styled.length))
        let stateColors = skin["state_colors"] as? [String: String] ?? [:]
        let statusRange = (text.stringValue as NSString).range(of: item["status"] as? String ?? "")
        if statusRange.location != NSNotFound {
            styled.addAttribute(.foregroundColor, value: color(stateColors[item["state"] as? String ?? "unknown"] ?? "#44536A"), range: statusRange)
        }
        if item["context_highlighted"] as? Bool == true, let summary = item["context_summary"] as? String {
            let range = (text.stringValue as NSString).range(of: summary, options: .backwards)
            if range.location != NSNotFound {
                styled.addAttributes([.foregroundColor: color(colors["accent"] ?? "#3155C6"),
                    .backgroundColor: color(colors["selection"] ?? "#EDF3FF"),
                    .font: NSFont.boldSystemFont(ofSize: text.font!.pointSize)], range: range)
            }
        }
        text.attributedStringValue = styled
        let cell = NSTableCellView()
        text.translatesAutoresizingMaskIntoConstraints = false; cell.addSubview(text); cell.textField = text
        let padding = CGFloat(skin["padding"] as? Int ?? 10)
        NSLayoutConstraint.activate([text.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: padding),
            text.trailingAnchor.constraint(equalTo: cell.trailingAnchor, constant: -padding),
            text.topAnchor.constraint(equalTo: cell.topAnchor, constant: padding),
            text.bottomAnchor.constraint(lessThanOrEqualTo: cell.bottomAnchor, constant: -padding)])
        return cell
    }
    func tableViewSelectionDidChange(_ notification: Notification) { if !rendering { updateSelection() } }
    func updateSelection() {
        let next = selected?["detail"] as? String ?? tr("세션을 선택하면 요청, 답변, 출처를 볼 수 있습니다.", "Select a session to read its prompt, answer and source details.")
        if detail.string != next { detail.string = next }
        for key in ["open", "pin", "hide", "dismiss", "copy", "connect", "disconnect"] { buttons[key]?.isEnabled = selected != nil }
        buttons["open"]?.isEnabled = selected != nil && view["busy"] as? Bool != true && selected?["dismissed"] as? Bool != true
        buttons["pin"]?.title = selected?["pinned"] as? Bool == true ? tr("고정 해제", "Unpin") : tr("고정", "Pin")
        buttons["hide"]?.title = selected?["hidden"] as? Bool == true ? tr("숨김 해제", "Unhide") : tr("숨기기", "Hide")
        buttons["dismiss"]?.title = selected?["dismissed"] as? Bool == true ? tr("다시 보기", "Restore") : tr("waid에서 보지 않기", "Do not show in waid")
        buttons["disconnect"]?.isEnabled = selected?["connected"] as? Bool == true
    }
    func controlTextDidChange(_ notification: Notification) { if !rendering { request("search", value: search.stringValue) } }
    @objc func filterChanged(_ sender: NSPopUpButton) {
        if sender === status { request("state", value: ["", "waiting", "working", "error", "idle", "done", "unknown"][sender.indexOfSelectedItem]) }
        else { request("agent", value: sender.indexOfSelectedItem == 0 ? "" : sender.titleOfSelectedItem ?? "") }
    }
    @objc func opacityChanged() { request("opacity", value: String(opacity.integerValue)) }
    @objc func languageChanged() { request("language", value: language.indexOfSelectedItem == 0 ? "ko" : "en") }
    @objc func clicked(_ sender: NSButton) {
        let action = sender.identifier!.rawValue
        switch action {
        case "copy":
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(selected?["prompt"] as? String ?? "", forType: .string)
        case "connect": request(action, value: (NSPasteboard.general.string(forType: .string) ?? "").trimmingCharacters(in: .whitespacesAndNewlines))
        case "editor": showEditor()
        case "preview", "template":
            request(action, value: editor.string)
            if let error = view["notice"] as? String, !error.isEmpty { showError(error, ok: tr("확인", "OK")) }
        case "day": editor.string = daylight
        case "night": editor.string = midnight
        case "import": importTemplate()
        case "export": exportTemplate()
        default: request(action)
        }
    }
    @objc func openSession() { if selected != nil { request("open") } }
    @objc func find() { showWindow(); window.makeFirstResponder(search) }
    @objc func hideApp() { NSApp.hide(nil) }
    @objc func showWindow() { window.deminiaturize(nil); window.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true) }
    @objc func quit() {
        request("save")
        if let error = view["notice"] as? String, !error.isEmpty { showError(error, ok: tr("확인", "OK")); return }
        NSApp.stop(nil)
        NSApp.postEvent(NSEvent.otherEvent(with: .applicationDefined, location: .zero, modifierFlags: [], timestamp: 0, windowNumber: 0, context: nil, subtype: 0, data1: 0, data2: 0)!, atStart: false)
    }
    func windowShouldClose(_ sender: NSWindow) -> Bool {
        if sender === editorWindow { request("cancel_preview") }
        sender.orderOut(nil); return false
    }
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool { showWindow(); return true }
    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply { quit(); return .terminateCancel }
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }
    @objc func showLicense() { let alert = NSAlert(); alert.messageText = tr("폰트 라이선스", "Font license"); alert.informativeText = fontLicense; alert.addButton(withTitle: tr("확인", "OK")); alert.runModal() }

    func showEditor() {
        if editorWindow == nil {
            let panel = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 700, height: 600), styleMask: [.titled, .closable, .resizable], backing: .buffered, defer: false)
            panel.title = tr("템플릿 — 복제, 편집, 미리보기, 적용", "Templates — duplicate, edit, preview, apply"); panel.isReleasedWhenClosed = false; panel.delegate = self
            editor.isRichText = false; editor.isAutomaticQuoteSubstitutionEnabled = false
            editor.isAutomaticDashSubstitutionEnabled = false
            editor.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
            editor.setAccessibilityLabel(tr("템플릿 JSON 편집기", "Template JSON editor"))
            let bar = stack([button(tr("밝은 테마", "Daylight"), "day"), button(tr("어두운 테마", "Midnight"), "night"), button(tr("가져오기…", "Import…"), "import"), button(tr("내보내기…", "Export…"), "export"), button(tr("미리보기", "Preview"), "preview"), button(tr("적용", "Apply"), "template")])
            let root = stack([scroll(editor), bar], vertical: true)
            root.translatesAutoresizingMaskIntoConstraints = false; panel.contentView!.addSubview(root)
            NSLayoutConstraint.activate([root.leadingAnchor.constraint(equalTo: panel.contentView!.leadingAnchor, constant: 10), root.trailingAnchor.constraint(equalTo: panel.contentView!.trailingAnchor, constant: -10), root.topAnchor.constraint(equalTo: panel.contentView!.topAnchor, constant: 10), root.bottomAnchor.constraint(equalTo: panel.contentView!.bottomAnchor, constant: -10), root.arrangedSubviews[0].widthAnchor.constraint(equalTo: root.widthAnchor)])
            editorWindow = panel
        }
        editor.string = view["template"] as? String ?? ""
        editorWindow!.makeKeyAndOrderFront(nil)
    }
    func importTemplate() {
        let panel = NSOpenPanel(); panel.allowedContentTypes = [.json]; panel.allowsMultipleSelection = false
        panel.title = tr("템플릿 가져오기", "Import template"); panel.prompt = tr("가져오기", "Import")
        if panel.runModal() == .OK, let url = panel.url {
            do {
                let handle = try FileHandle(forReadingFrom: url); defer { try? handle.close() }
                let data = try handle.read(upToCount: 128 * 1024 + 1) ?? Data()
                guard data.count <= 128 * 1024, let text = String(data: data, encoding: .utf8) else { throw CocoaError(.fileReadTooLarge) }
                editor.string = text.hasPrefix("\u{feff}") ? String(text.dropFirst()) : text
            } catch { showError(error.localizedDescription, ok: tr("확인", "OK")) }
        }
    }
    func exportTemplate() {
        let panel = NSSavePanel(); panel.nameFieldStringValue = "waid-template.json"
        panel.title = tr("템플릿 내보내기", "Export template"); panel.prompt = tr("내보내기", "Export")
        if panel.runModal() == .OK, let url = panel.url {
            do { try editor.string.write(to: url, atomically: true, encoding: .utf8) }
            catch { showError(error.localizedDescription, ok: tr("확인", "OK")) }
        }
    }
    func smoke() {
        func check(_ value: Bool, _ message: String = "", line: UInt = #line) {
            if !value { FileHandle.standardError.write(Data("AppKit smoke failed at App.swift:\(line)\n".utf8)) }
            precondition(value)
        }
        request("poll")
        if rows.isEmpty {
            smokeAttempts += 1
            check(smokeAttempts < 30, "Smoke collector did not deliver the fixture")
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { self.smoke() }
            return
        }
        check(window.isVisible && table.numberOfColumns == 1 && statusItem.menu!.items.count == 2)
        let rowID = rows[0]["id"] as! String
        if CommandLine.arguments.contains("--macos-smoke-restart") {
            check(rows[0]["pinned"] as? Bool == true)
            check(abs(window.alphaValue - 0.75) < 0.01)
            check(view["template"] as? String == midnight)
            check(view["language"] as? String == "en")
            request("pin", id: rowID)
            request("opacity", value: "100")
            request("template", value: daylight)
        }
        table.selectRowIndexes(IndexSet(integer: 0), byExtendingSelection: false)
        check(detail.string.contains("Mac fixture"))
        let originalPrompt = selected?["prompt"] as? String
        showEditor()
        let originalTemplate = editor.string
        language.selectItem(at: 0); languageChanged()
        check(view["language"] as? String == "ko" && search.accessibilityLabel() == "세션 검색")
        check(status.itemTitles[0] == "모든 상태" && buttons["copy"]?.title == "요청 복사")
        check(statusItem.menu?.items.first?.title == "waid 열기" && buttons["preview"]?.title == "미리보기")
        check(selected?["prompt"] as? String == originalPrompt && detail.string.contains("마지막 답변"))
        language.selectItem(at: 1); languageChanged()
        check(view["language"] as? String == "en" && search.accessibilityLabel() == "Search sessions")
        check(status.itemTitles[0] == "All statuses" && buttons["copy"]?.title == "Copy prompt")
        check(statusItem.menu?.items.first?.title == "Show waid" && buttons["preview"]?.title == "Preview")
        check(selected?["prompt"] as? String == originalPrompt && detail.string.contains("Last answer"))
        check(editor.string == originalTemplate)
        editorWindow!.performClose(nil)
        clicked(buttons["copy"]!)
        check(NSPasteboard.general.string(forType: .string)?.contains("Mac fixture") == true)
        request("pin", id: rowID); check(selected?["pinned"] as? Bool == true)
        request("dismiss", id: rowID); check(rows.isEmpty)
        request("all"); check(rows.count == 1 && rows[0]["dismissed"] as? Bool == true)
        table.selectRowIndexes(IndexSet(integer: 0), byExtendingSelection: false)
        request("dismiss", id: rowID); check(rows[0]["dismissed"] as? Bool == false)
        request("all"); request("pin", id: rowID)
        if let path = ProcessInfo.processInfo.environment["WAID_SMOKE_SCREENSHOT"] {
            for (file, template) in [(path, daylight), (path + "-night.png", midnight)] {
                request("preview", value: template)
                window.contentView!.layoutSubtreeIfNeeded()
                check(split.frame.height >= 330)
                let content = window.contentView!
                let bitmap = content.bitmapImageRepForCachingDisplay(in: content.bounds)!
                content.cacheDisplay(in: content.bounds, to: bitmap)
                try! bitmap.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: file))
            }
            request("cancel_preview")
        }
        search.stringValue = "mac smoke"
        controlTextDidChange(Notification(name: NSControl.textDidChangeNotification, object: search))
        check(search.stringValue == "mac smoke")
        check(rows.isEmpty)
        search.stringValue = ""
        controlTextDidChange(Notification(name: NSControl.textDidChangeNotification, object: search))
        request("top"); check(window.level == .floating)
        request("top"); request("opacity", value: "75")
        check(abs(window.alphaValue - 0.75) < 0.01)
        request("opacity", value: "100")
        window.performClose(nil); check(!window.isVisible)
        showWindow(); check(window.isVisible)
        showEditor(); check(editorWindow!.isVisible && !editor.string.isEmpty)
        editorWindow!.performClose(nil); check(!editorWindow!.isVisible)
        request("pin", id: rowID)
        request("opacity", value: "75")
        request("template", value: midnight)
        print("AppKit smoke passed: fixture collection, selection, Korean/English switching and persistence, clipboard, pin/dismiss/restore, window, menu bar, settings, template editor, hide/reopen")
        quit()
    }
}

func showError(_ text: String, ok: String = "OK") { let alert = NSAlert(); alert.messageText = "waid"; alert.informativeText = text; alert.alertStyle = .warning; alert.addButton(withTitle: ok); alert.runModal() }

@_cdecl("waid_app_run")
public func runApp(_ context: UnsafeMutableRawPointer?, _ callback: @escaping @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> UnsafePointer<CChar>?) {
    let app = NSApplication.shared
    app.setActivationPolicy(.regular)
    let delegate = WaidApp(context, callback)
    app.delegate = delegate
    withExtendedLifetime(delegate) { app.run() }
    delegate.timer?.invalidate()
    NSStatusBar.system.removeStatusItem(delegate.statusItem)
    app.delegate = nil
}

@_cdecl("waid_app_error")
public func appError(_ message: UnsafePointer<CChar>) { _ = NSApplication.shared; showError(String(cString: message)) }
