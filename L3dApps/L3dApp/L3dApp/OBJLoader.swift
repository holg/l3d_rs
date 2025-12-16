import SceneKit
import Foundation

class OBJLoader {
    static func loadOBJ(from data: Data, name: String) -> SCNNode? {
        guard let objString = String(data: data, encoding: .utf8) else { return nil }
        return parseOBJ(objString, name: name)
    }

    private static func parseOBJ(_ content: String, name: String) -> SCNNode? {
        var vertices: [SCNVector3] = []
        var normals: [SCNVector3] = []
        var texCoords: [CGPoint] = []
        var faces: [(vertexIndices: [Int], normalIndices: [Int], texCoordIndices: [Int])] = []

        let lines = content.components(separatedBy: .newlines)

        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.isEmpty || trimmed.hasPrefix("#") { continue }

            let parts = trimmed.split(separator: " ", omittingEmptySubsequences: true)
            guard !parts.isEmpty else { continue }

            let command = String(parts[0])

            switch command {
            case "v":
                if parts.count >= 4 {
                    let x = Float(parts[1]) ?? 0
                    let y = Float(parts[2]) ?? 0
                    let z = Float(parts[3]) ?? 0
                    vertices.append(SCNVector3(x, y, z))
                }

            case "vn":
                if parts.count >= 4 {
                    let x = Float(parts[1]) ?? 0
                    let y = Float(parts[2]) ?? 0
                    let z = Float(parts[3]) ?? 0
                    normals.append(SCNVector3(x, y, z))
                }

            case "vt":
                if parts.count >= 3 {
                    let u = CGFloat(Float(parts[1]) ?? 0)
                    let v = CGFloat(Float(parts[2]) ?? 0)
                    texCoords.append(CGPoint(x: u, y: v))
                }

            case "f":
                var vertexIndices: [Int] = []
                var normalIndices: [Int] = []
                var texCoordIndices: [Int] = []

                for i in 1..<parts.count {
                    let indices = parseFaceVertex(String(parts[i]))
                    if let vi = indices.vertex { vertexIndices.append(vi - 1) }
                    if let ti = indices.texCoord { texCoordIndices.append(ti - 1) }
                    if let ni = indices.normal { normalIndices.append(ni - 1) }
                }

                if !vertexIndices.isEmpty {
                    faces.append((vertexIndices, normalIndices, texCoordIndices))
                }

            default:
                break
            }
        }

        guard !vertices.isEmpty, !faces.isEmpty else { return nil }

        return buildGeometry(vertices: vertices, normals: normals, texCoords: texCoords, faces: faces, name: name)
    }

    private static func parseFaceVertex(_ spec: String) -> (vertex: Int?, texCoord: Int?, normal: Int?) {
        let parts = spec.split(separator: "/", omittingEmptySubsequences: false)
        let vertex = parts.count > 0 ? Int(parts[0]) : nil
        let texCoord = parts.count > 1 && !parts[1].isEmpty ? Int(parts[1]) : nil
        let normal = parts.count > 2 ? Int(parts[2]) : nil
        return (vertex, texCoord, normal)
    }

    private static func buildGeometry(
        vertices: [SCNVector3],
        normals: [SCNVector3],
        texCoords: [CGPoint],
        faces: [(vertexIndices: [Int], normalIndices: [Int], texCoordIndices: [Int])],
        name: String
    ) -> SCNNode {
        var flatVertices: [SCNVector3] = []
        var flatNormals: [SCNVector3] = []
        var flatTexCoords: [CGPoint] = []
        var indices: [Int32] = []
        var currentIndex: Int32 = 0

        for face in faces {
            let vertexIndices = face.vertexIndices
            let normalIndices = face.normalIndices
            let texCoordIndices = face.texCoordIndices

            guard vertexIndices.count >= 3 else { continue }

            for i in 1..<(vertexIndices.count - 1) {
                let triIndices = [0, i, i + 1]

                for ti in triIndices {
                    let vi = vertexIndices[ti]
                    flatVertices.append(vi >= 0 && vi < vertices.count ? vertices[vi] : SCNVector3(0, 0, 0))

                    if !normalIndices.isEmpty && ti < normalIndices.count {
                        let ni = normalIndices[ti]
                        flatNormals.append(ni >= 0 && ni < normals.count ? normals[ni] : SCNVector3(0, 1, 0))
                    }

                    if !texCoordIndices.isEmpty && ti < texCoordIndices.count {
                        let tci = texCoordIndices[ti]
                        flatTexCoords.append(tci >= 0 && tci < texCoords.count ? texCoords[tci] : CGPoint(x: 0, y: 0))
                    }

                    indices.append(currentIndex)
                    currentIndex += 1
                }
            }
        }

        let vertexSource = SCNGeometrySource(vertices: flatVertices)
        var sources = [vertexSource]

        if !flatNormals.isEmpty && flatNormals.count == flatVertices.count {
            sources.append(SCNGeometrySource(normals: flatNormals))
        }

        if !flatTexCoords.isEmpty && flatTexCoords.count == flatVertices.count {
            let texCoordData = flatTexCoords.withUnsafeBufferPointer { Data(buffer: $0) }
            let texCoordSource = SCNGeometrySource(
                data: texCoordData,
                semantic: .texcoord,
                vectorCount: flatTexCoords.count,
                usesFloatComponents: true,
                componentsPerVector: 2,
                bytesPerComponent: MemoryLayout<CGFloat>.size,
                dataOffset: 0,
                dataStride: MemoryLayout<CGPoint>.size
            )
            sources.append(texCoordSource)
        }

        let indexData = Data(bytes: indices, count: indices.count * MemoryLayout<Int32>.size)
        let element = SCNGeometryElement(
            data: indexData,
            primitiveType: .triangles,
            primitiveCount: indices.count / 3,
            bytesPerIndex: MemoryLayout<Int32>.size
        )

        let geometry = SCNGeometry(sources: sources, elements: [element])

        let material = SCNMaterial()
        #if os(macOS)
        material.diffuse.contents = NSColor(red: 0.8, green: 0.8, blue: 0.8, alpha: 1.0)
        #else
        material.diffuse.contents = UIColor(red: 0.8, green: 0.8, blue: 0.8, alpha: 1.0)
        #endif
        material.lightingModel = .physicallyBased
        material.roughness.contents = 0.5
        material.metalness.contents = 0.1
        material.isDoubleSided = true
        geometry.materials = [material]

        let node = SCNNode(geometry: geometry)
        node.name = name
        return node
    }

    static func applyTransform(_ node: SCNNode, matrix: [Float]) {
        guard matrix.count == 16 else { return }

        let m = SCNMatrix4(
            m11: CGFloat(matrix[0]), m12: CGFloat(matrix[1]), m13: CGFloat(matrix[2]), m14: CGFloat(matrix[3]),
            m21: CGFloat(matrix[4]), m22: CGFloat(matrix[5]), m23: CGFloat(matrix[6]), m24: CGFloat(matrix[7]),
            m31: CGFloat(matrix[8]), m32: CGFloat(matrix[9]), m33: CGFloat(matrix[10]), m34: CGFloat(matrix[11]),
            m41: CGFloat(matrix[12]), m42: CGFloat(matrix[13]), m43: CGFloat(matrix[14]), m44: CGFloat(matrix[15])
        )

        node.transform = m
    }
}
