//
//  CollagePreviewView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI
import Photos

struct CollagePreviewView: View {
    @Environment(\.dismiss) private var dismiss

    let assets: [PHAsset]
    var onSaveComplete: (() -> Void)?

    @State private var images: [String: UIImage] = [:]
    @State private var isLoadingImages = true
    @State private var isSaving = false
    @State private var showSaveSuccess = false
    @State private var saveError: String?

    private let collageSize: CGFloat = 1080 // Output image size

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                Spacer()

                // Collage preview
                CollageGridView(assets: assets, images: images)
                    .aspectRatio(1, contentMode: .fit)
                    .padding()
                    .background(Color(.systemGray6))

                Spacer()

                // Save button
                Button(action: saveCollage) {
                    HStack {
                        if isSaving {
                            ProgressView()
                                .tint(.white)
                        } else {
                            Image(systemName: "square.and.arrow.down")
                        }
                        Text(isSaving ? "Saving..." : "Save to Photos")
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(isLoadingImages || isSaving ? Color.gray : Color.blue)
                    .foregroundColor(.white)
                    .cornerRadius(12)
                }
                .disabled(isLoadingImages || isSaving)
                .padding()
            }
            .navigationTitle("Collage Preview")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("Back") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                loadFullResolutionImages()
            }
            .alert("Collage Saved!", isPresented: $showSaveSuccess) {
                Button("Done") {
                    if let onSaveComplete = onSaveComplete {
                        onSaveComplete()
                    } else {
                        dismiss()
                    }
                }
            } message: {
                Text("Your collage has been saved to your photo library.")
            }
            .alert("Save Error", isPresented: .init(
                get: { saveError != nil },
                set: { if !$0 { saveError = nil } }
            )) {
                Button("OK") {
                    saveError = nil
                }
            } message: {
                if let error = saveError {
                    Text(error)
                }
            }
        }
    }

    private func loadFullResolutionImages() {
        isLoadingImages = true
        let imageManager = PHImageManager.default()
        let options = PHImageRequestOptions()
        options.deliveryMode = .highQualityFormat
        options.isSynchronous = false
        options.isNetworkAccessAllowed = true

        let targetSize = CGSize(width: collageSize, height: collageSize)
        var loadedCount = 0

        for asset in assets {
            imageManager.requestImage(
                for: asset,
                targetSize: targetSize,
                contentMode: .aspectFill,
                options: options
            ) { result, info in
                if let image = result {
                    DispatchQueue.main.async {
                        images[asset.localIdentifier] = image
                        loadedCount += 1
                        if loadedCount >= assets.count {
                            isLoadingImages = false
                        }
                    }
                }
            }
        }
    }

    private func saveCollage() {
        isSaving = true

        Task {
            do {
                let collageImage = try await renderCollage()
                try await saveImageToPhotoLibrary(collageImage)

                await MainActor.run {
                    isSaving = false
                    showSaveSuccess = true
                }
            } catch {
                await MainActor.run {
                    isSaving = false
                    saveError = error.localizedDescription
                }
            }
        }
    }

    private func renderCollage() async throws -> UIImage {
        let size = CGSize(width: collageSize, height: collageSize)
        let layout = calculateLayout(photoCount: assets.count, canvasSize: size)

        let renderer = UIGraphicsImageRenderer(size: size)
        let collageImage = renderer.image { context in
            // Fill background
            UIColor.black.setFill()
            context.fill(CGRect(origin: .zero, size: size))

            // Draw each image in its grid position
            for (index, frame) in layout.enumerated() {
                guard index < assets.count else { break }
                let asset = assets[index]

                if let image = images[asset.localIdentifier] {
                    // Calculate aspect fill frame
                    let imageRect = aspectFillRect(for: image.size, in: frame)
                    image.draw(in: imageRect)
                }
            }
        }

        return collageImage
    }

    private func calculateLayout(photoCount: Int, canvasSize: CGSize) -> [CGRect] {
        let spacing: CGFloat = 4
        var frames: [CGRect] = []

        // Determine grid dimensions based on photo count
        let (rows, cols) = gridDimensions(for: photoCount)

        let cellWidth = (canvasSize.width - spacing * CGFloat(cols + 1)) / CGFloat(cols)
        let cellHeight = (canvasSize.height - spacing * CGFloat(rows + 1)) / CGFloat(rows)

        for i in 0..<photoCount {
            let row = i / cols
            let col = i % cols

            let x = spacing + CGFloat(col) * (cellWidth + spacing)
            let y = spacing + CGFloat(row) * (cellHeight + spacing)

            frames.append(CGRect(x: x, y: y, width: cellWidth, height: cellHeight))
        }

        return frames
    }

    private func gridDimensions(for count: Int) -> (rows: Int, cols: Int) {
        switch count {
        case 2: return (1, 2)
        case 3: return (1, 3)
        case 4: return (2, 2)
        case 5, 6: return (2, 3)
        case 7, 8, 9: return (3, 3)
        default: return (1, 1)
        }
    }

    private func aspectFillRect(for imageSize: CGSize, in targetRect: CGRect) -> CGRect {
        let imageAspect = imageSize.width / imageSize.height
        let targetAspect = targetRect.width / targetRect.height

        var drawRect: CGRect

        if imageAspect > targetAspect {
            // Image is wider - fit height, crop width
            let drawHeight = targetRect.height
            let drawWidth = drawHeight * imageAspect
            let offsetX = (drawWidth - targetRect.width) / 2
            drawRect = CGRect(
                x: targetRect.origin.x - offsetX,
                y: targetRect.origin.y,
                width: drawWidth,
                height: drawHeight
            )
        } else {
            // Image is taller - fit width, crop height
            let drawWidth = targetRect.width
            let drawHeight = drawWidth / imageAspect
            let offsetY = (drawHeight - targetRect.height) / 2
            drawRect = CGRect(
                x: targetRect.origin.x,
                y: targetRect.origin.y - offsetY,
                width: drawWidth,
                height: drawHeight
            )
        }

        return drawRect
    }

    private func saveImageToPhotoLibrary(_ image: UIImage) async throws {
        try await PHPhotoLibrary.shared().performChanges {
            PHAssetChangeRequest.creationRequestForAsset(from: image)
        }
    }
}

struct CollageGridView: View {
    let assets: [PHAsset]
    let images: [String: UIImage]

    private var gridLayout: (rows: Int, cols: Int) {
        switch assets.count {
        case 2: return (1, 2)
        case 3: return (1, 3)
        case 4: return (2, 2)
        case 5, 6: return (2, 3)
        case 7, 8, 9: return (3, 3)
        default: return (1, 1)
        }
    }

    var body: some View {
        GeometryReader { geometry in
            let spacing: CGFloat = 2
            let cols = gridLayout.cols
            let rows = gridLayout.rows
            let cellWidth = (geometry.size.width - spacing * CGFloat(cols + 1)) / CGFloat(cols)
            let cellHeight = (geometry.size.height - spacing * CGFloat(rows + 1)) / CGFloat(rows)

            ZStack {
                Color.black

                ForEach(Array(assets.enumerated()), id: \.element.localIdentifier) { index, asset in
                    let row = index / cols
                    let col = index % cols
                    let x = spacing + CGFloat(col) * (cellWidth + spacing)
                    let y = spacing + CGFloat(row) * (cellHeight + spacing)

                    if let image = images[asset.localIdentifier] {
                        Image(uiImage: image)
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                            .frame(width: cellWidth, height: cellHeight)
                            .clipped()
                            .position(x: x + cellWidth / 2, y: y + cellHeight / 2)
                    } else {
                        Rectangle()
                            .fill(Color.gray.opacity(0.5))
                            .frame(width: cellWidth, height: cellHeight)
                            .overlay(ProgressView().tint(.white))
                            .position(x: x + cellWidth / 2, y: y + cellHeight / 2)
                    }
                }
            }
        }
    }
}

#Preview {
    CollagePreviewView(assets: [])
}
