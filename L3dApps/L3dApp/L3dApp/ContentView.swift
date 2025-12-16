import SwiftUI
import L3dKit
import UniformTypeIdentifiers
import SceneKit

struct ContentView: View {
    @State private var l3dFile: L3dFile?
    @State private var errorMessage: String?
    @State private var isImporting = false
    @State private var currentFileName: String = ""
    @State private var isTargeted = false

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                if let l3dFile = l3dFile {
                    L3DSceneView(l3dFile: l3dFile)
                } else {
                    emptyStateView
                }
            }
            .navigationTitle(currentFileName.isEmpty ? "L3D Viewer" : currentFileName)
            #if os(macOS)
            .navigationSubtitle(l3dFile != nil ? "\(l3dFile!.getPartCount()) parts" : "")
            #endif
            .toolbar { toolbarContent }
            .alert("Error", isPresented: .constant(errorMessage != nil)) {
                Button("OK") { errorMessage = nil }
            } message: {
                Text(errorMessage ?? "")
            }
            .onDrop(of: [.fileURL], isTargeted: $isTargeted, perform: handleDrop)
            .fileImporter(isPresented: $isImporting, allowedContentTypes: [.l3d], allowsMultipleSelection: false, onCompletion: handleFileImport)
            .onReceive(NotificationCenter.default.publisher(for: .openFile)) { _ in
                isImporting = true
            }
            .onReceive(NotificationCenter.default.publisher(for: .openExternalFile)) { notification in
                if let url = notification.object as? URL {
                    loadExternalFile(url: url)
                }
            }
        }
    }

    private var emptyStateView: some View {
        VStack(spacing: 24) {
            Image(systemName: "cube.transparent")
                .font(.system(size: 80))
                .foregroundStyle(.tertiary)

            VStack(spacing: 8) {
                Text("No L3D File Loaded")
                    .font(.title2)
                    .fontWeight(.medium)

                Text("Open an L3D file or drag and drop one here")
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }

            Button {
                isImporting = true
            } label: {
                Label("Open File", systemImage: "folder")
            }
            .buttonStyle(.borderedProminent)

            if isTargeted {
                Text("Drop file here")
                    .font(.headline)
                    .foregroundColor(.accentColor)
                    .padding()
                    .background(
                        RoundedRectangle(cornerRadius: 12)
                            .strokeBorder(style: StrokeStyle(lineWidth: 2, dash: [8]))
                            .foregroundColor(.accentColor)
                    )
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItemGroup(placement: .primaryAction) {
            Button {
                isImporting = true
            } label: {
                Label("Open", systemImage: "folder")
            }
        }
    }

    private func handleFileImport(_ result: Result<[URL], Error>) {
        switch result {
        case .success(let urls):
            guard let url = urls.first else { return }
            loadFile(url: url)
        case .failure(let error):
            errorMessage = error.localizedDescription
        }
    }

    private func handleDrop(providers: [NSItemProvider]) -> Bool {
        guard let provider = providers.first else { return false }

        provider.loadItem(forTypeIdentifier: UTType.fileURL.identifier, options: nil) { item, error in
            guard let data = item as? Data,
                  let url = URL(dataRepresentation: data, relativeTo: nil) else {
                return
            }

            DispatchQueue.main.async {
                loadFile(url: url)
            }
        }
        return true
    }

    private func loadFile(url: URL) {
        guard url.startAccessingSecurityScopedResource() else {
            errorMessage = "Cannot access file"
            return
        }
        defer { url.stopAccessingSecurityScopedResource() }
        loadFileContents(url: url)
    }

    private func loadExternalFile(url: URL) {
        loadFileContents(url: url)
    }

    private func loadFileContents(url: URL) {
        currentFileName = url.lastPathComponent

        do {
            let data = try Data(contentsOf: url)
            let file = try L3dFile(data: data)
            l3dFile = file
        } catch {
            errorMessage = "Failed to parse L3D file: \(error.localizedDescription)"
        }
    }
}

// MARK: - 3D Scene View

struct L3DSceneView: View {
    let l3dFile: L3dFile

    var body: some View {
        L3DSceneKitView(l3dFile: l3dFile)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

#if os(macOS)
struct L3DSceneKitView: NSViewRepresentable {
    let l3dFile: L3dFile

    func makeNSView(context: Context) -> SCNView {
        let sceneView = SCNView()
        sceneView.allowsCameraControl = true
        sceneView.autoenablesDefaultLighting = true
        sceneView.backgroundColor = NSColor(white: 0.15, alpha: 1)
        sceneView.scene = buildScene()
        return sceneView
    }

    func updateNSView(_ nsView: SCNView, context: Context) {
        nsView.scene = buildScene()
    }

    private func buildScene() -> SCNScene {
        let scene = SCNScene()

        // Create luminaire node that contains all parts
        let luminaireNode = SCNNode()
        luminaireNode.name = "luminaire"

        let parts = l3dFile.getParts()
        let assets = l3dFile.getAssets()

        for part in parts {
            // Find matching OBJ asset
            let partFilename = (part.path as NSString).lastPathComponent.lowercased()
            if let asset = assets.first(where: {
                let assetFilename = ($0.name as NSString).lastPathComponent.lowercased()
                return assetFilename == partFilename ||
                       $0.name.lowercased().contains(partFilename) ||
                       part.path.lowercased().contains(assetFilename)
            }) {
                let objData = Data(asset.content)
                if let partNode = OBJLoader.loadOBJ(from: objData, name: part.name) {
                    // Apply transformation matrix
                    OBJLoader.applyTransform(partNode, matrix: part.transform)
                    luminaireNode.addChildNode(partNode)
                }
            }
        }

        // If no OBJ parts loaded, create placeholder boxes
        if luminaireNode.childNodes.isEmpty {
            for (index, part) in parts.enumerated() {
                let box = SCNBox(width: 0.05, height: 0.05, length: 0.05, chamferRadius: 0.005)
                box.firstMaterial?.diffuse.contents = NSColor.systemBlue
                let node = SCNNode(geometry: box)
                OBJLoader.applyTransform(node, matrix: part.transform)
                luminaireNode.addChildNode(node)
            }
        }

        scene.rootNode.addChildNode(luminaireNode)

        // Auto-fit camera to content
        let (minBound, maxBound) = luminaireNode.boundingBox
        let center = SCNVector3(
            (minBound.x + maxBound.x) / 2,
            (minBound.y + maxBound.y) / 2,
            (minBound.z + maxBound.z) / 2
        )
        let size = max(maxBound.x - minBound.x, max(maxBound.y - minBound.y, maxBound.z - minBound.z))
        let distance = max(size * 2, 0.5)

        let cameraNode = SCNNode()
        cameraNode.camera = SCNCamera()
        cameraNode.camera?.automaticallyAdjustsZRange = true
        cameraNode.position = SCNVector3(
            CGFloat(center.x),
            CGFloat(center.y) + CGFloat(distance) * 0.3,
            CGFloat(center.z) + CGFloat(distance)
        )
        cameraNode.look(at: center)
        scene.rootNode.addChildNode(cameraNode)

        // Add lights
        let lightNode = SCNNode()
        lightNode.light = SCNLight()
        lightNode.light?.type = .omni
        lightNode.light?.intensity = 1000
        lightNode.position = SCNVector3(CGFloat(distance), CGFloat(distance), CGFloat(distance))
        scene.rootNode.addChildNode(lightNode)

        let ambientLight = SCNNode()
        ambientLight.light = SCNLight()
        ambientLight.light?.type = .ambient
        ambientLight.light?.intensity = 300
        scene.rootNode.addChildNode(ambientLight)

        return scene
    }
}
#else
struct L3DSceneKitView: UIViewRepresentable {
    let l3dFile: L3dFile

    func makeUIView(context: Context) -> SCNView {
        let sceneView = SCNView()
        sceneView.allowsCameraControl = true
        sceneView.autoenablesDefaultLighting = true
        sceneView.backgroundColor = UIColor(white: 0.15, alpha: 1)
        sceneView.scene = buildScene()
        return sceneView
    }

    func updateUIView(_ uiView: SCNView, context: Context) {
        uiView.scene = buildScene()
    }

    private func buildScene() -> SCNScene {
        let scene = SCNScene()
        let luminaireNode = SCNNode()

        let parts = l3dFile.getParts()
        let assets = l3dFile.getAssets()

        for part in parts {
            let partFilename = (part.path as NSString).lastPathComponent.lowercased()
            if let asset = assets.first(where: {
                ($0.name as NSString).lastPathComponent.lowercased() == partFilename
            }) {
                let objData = Data(asset.content)
                if let partNode = OBJLoader.loadOBJ(from: objData, name: part.name) {
                    OBJLoader.applyTransform(partNode, matrix: part.transform)
                    luminaireNode.addChildNode(partNode)
                }
            }
        }

        scene.rootNode.addChildNode(luminaireNode)

        let cameraNode = SCNNode()
        cameraNode.camera = SCNCamera()
        cameraNode.position = SCNVector3(0, 0.5, 1)
        cameraNode.look(at: SCNVector3(0, 0, 0))
        scene.rootNode.addChildNode(cameraNode)

        return scene
    }
}
#endif

// MARK: - UTType

extension UTType {
    static var l3d: UTType {
        UTType(filenameExtension: "l3d", conformingTo: .archive) ?? UTType(exportedAs: "io.gldf.l3d")
    }
}
