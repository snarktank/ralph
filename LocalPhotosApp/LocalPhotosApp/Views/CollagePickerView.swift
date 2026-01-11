//
//  CollagePickerView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI
import Photos

struct CollagePickerView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var photoService = PhotoLibraryService()

    @State private var selectedAssetIds: Set<String> = []
    @State private var showCollagePreview = false
    @State private var selectedAssets: [PHAsset] = []
    @State private var collageSaved = false

    private let minPhotos = 2
    private let maxPhotos = 9

    private let columns = [
        GridItem(.flexible(), spacing: 2),
        GridItem(.flexible(), spacing: 2),
        GridItem(.flexible(), spacing: 2)
    ]

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Selection count header
                HStack {
                    Text("\(selectedAssetIds.count) selected")
                        .font(.subheadline)
                        .foregroundColor(.secondary)

                    Spacer()

                    Text("Select \(minPhotos)-\(maxPhotos) photos")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding(.horizontal)
                .padding(.vertical, 8)
                .background(Color(.systemGray6))

                if photoService.isLoading {
                    Spacer()
                    ProgressView("Loading photos...")
                    Spacer()
                } else if photoService.isEmpty {
                    Spacer()
                    ContentUnavailableView(
                        "No Photos",
                        systemImage: "photo.on.rectangle",
                        description: Text("Your photo library is empty")
                    )
                    Spacer()
                } else {
                    ScrollView {
                        LazyVGrid(columns: columns, spacing: 2) {
                            ForEach(photoService.assets.filter { $0.mediaType == .image }, id: \.localIdentifier) { asset in
                                CollagePickerThumbnail(
                                    asset: asset,
                                    isSelected: selectedAssetIds.contains(asset.localIdentifier),
                                    selectionNumber: selectionNumber(for: asset)
                                )
                                .onTapGesture {
                                    toggleSelection(asset)
                                }
                            }
                        }
                    }
                }
            }
            .navigationTitle("Select Photos")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .topBarTrailing) {
                    Button("Next") {
                        prepareSelectedAssets()
                    }
                    .disabled(selectedAssetIds.count < minPhotos)
                    .fontWeight(.semibold)
                }
            }
            .onAppear {
                photoService.fetchAllAssets()
            }
            .fullScreenCover(isPresented: $showCollagePreview) {
                CollagePreviewView(assets: selectedAssets, onSaveComplete: {
                    collageSaved = true
                    showCollagePreview = false
                })
            }
            .onChange(of: collageSaved) { _, saved in
                if saved {
                    dismiss()
                }
            }
        }
    }

    private func toggleSelection(_ asset: PHAsset) {
        let id = asset.localIdentifier

        if selectedAssetIds.contains(id) {
            selectedAssetIds.remove(id)
        } else if selectedAssetIds.count < maxPhotos {
            selectedAssetIds.insert(id)
        }
    }

    private func selectionNumber(for asset: PHAsset) -> Int? {
        guard selectedAssetIds.contains(asset.localIdentifier) else { return nil }

        // Get the order based on when the asset was selected
        // For simplicity, we'll just show the position in the sorted set
        let sortedIds = Array(selectedAssetIds).sorted()
        return (sortedIds.firstIndex(of: asset.localIdentifier) ?? 0) + 1
    }

    private func prepareSelectedAssets() {
        // Fetch the actual PHAsset objects for selected IDs
        let fetchOptions = PHFetchOptions()
        fetchOptions.predicate = NSPredicate(format: "localIdentifier IN %@", Array(selectedAssetIds))

        let result = PHAsset.fetchAssets(with: fetchOptions)

        var assets: [PHAsset] = []
        result.enumerateObjects { asset, _, _ in
            assets.append(asset)
        }

        selectedAssets = assets
        showCollagePreview = true
    }
}

struct CollagePickerThumbnail: View {
    let asset: PHAsset
    let isSelected: Bool
    let selectionNumber: Int?

    @State private var image: UIImage?

    private static let imageManager = PHCachingImageManager()

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                if let image = image {
                    Image(uiImage: image)
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(width: geometry.size.width, height: geometry.size.height)
                        .clipped()
                } else {
                    Rectangle()
                        .fill(Color.gray.opacity(0.3))
                }

                // Selection overlay
                if isSelected {
                    Rectangle()
                        .fill(Color.blue.opacity(0.3))

                    VStack {
                        HStack {
                            Spacer()
                            if let number = selectionNumber {
                                Text("\(number)")
                                    .font(.caption)
                                    .fontWeight(.bold)
                                    .foregroundColor(.white)
                                    .frame(width: 24, height: 24)
                                    .background(Circle().fill(Color.blue))
                                    .padding(6)
                            }
                        }
                        Spacer()
                    }
                }
            }
        }
        .aspectRatio(1, contentMode: .fit)
        .onAppear {
            loadThumbnail()
        }
    }

    private func loadThumbnail() {
        let options = PHImageRequestOptions()
        options.deliveryMode = .opportunistic
        options.isNetworkAccessAllowed = true
        options.resizeMode = .fast

        let scale = UIScreen.main.scale
        let targetSize = CGSize(width: 120 * scale, height: 120 * scale)

        Self.imageManager.requestImage(
            for: asset,
            targetSize: targetSize,
            contentMode: .aspectFill,
            options: options
        ) { result, info in
            if let result = result {
                DispatchQueue.main.async {
                    self.image = result
                }
            }
        }
    }
}

#Preview {
    CollagePickerView()
}
