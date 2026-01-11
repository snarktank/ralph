//
//  GalleryView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI
import Photos

struct GalleryView: View {
    @StateObject private var permissionManager = PhotoLibraryPermissionManager()

    var body: some View {
        NavigationStack {
            Group {
                switch permissionManager.authorizationStatus {
                case .notDetermined:
                    PermissionRequestView(permissionManager: permissionManager)
                case .authorized, .limited:
                    AuthorizedGalleryContent(isLimited: permissionManager.authorizationStatus == .limited)
                case .denied, .restricted:
                    PermissionDeniedView(permissionManager: permissionManager)
                @unknown default:
                    Text("Unknown permission state")
                }
            }
            .navigationTitle("Gallery")
        }
    }
}

struct PermissionRequestView: View {
    @ObservedObject var permissionManager: PhotoLibraryPermissionManager

    var body: some View {
        VStack(spacing: 20) {
            Image(systemName: "photo.on.rectangle.angled")
                .font(.system(size: 60))
                .foregroundColor(.blue)

            Text("Access Your Photos")
                .font(.title2)
                .fontWeight(.semibold)

            Text(permissionManager.statusMessage)
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            Button("Allow Photo Access") {
                Task {
                    await permissionManager.requestAuthorization()
                }
            }
            .buttonStyle(.borderedProminent)
        }
        .padding()
    }
}

struct PermissionDeniedView: View {
    @ObservedObject var permissionManager: PhotoLibraryPermissionManager

    var body: some View {
        VStack(spacing: 20) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 60))
                .foregroundColor(.orange)

            Text("Photo Access Required")
                .font(.title2)
                .fontWeight(.semibold)

            Text(permissionManager.statusMessage)
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            Button("Open Settings") {
                if let settingsURL = URL(string: UIApplication.openSettingsURLString) {
                    UIApplication.shared.open(settingsURL)
                }
            }
            .buttonStyle(.borderedProminent)
        }
        .padding()
    }
}

struct AuthorizedGalleryContent: View {
    let isLimited: Bool

    @StateObject private var photoLibraryService = PhotoLibraryService()
    @StateObject private var albumService = AlbumService()
    @State private var selectedPhotoIndex: Int = 0
    @State private var isShowingPhotoDetail: Bool = false
    @State private var isSelectionMode: Bool = false
    @State private var selectedAssetIds: Set<String> = []
    @State private var isShowingAlbumPicker: Bool = false
    @State private var isAddingToAlbum: Bool = false

    private let columns = [
        GridItem(.flexible(), spacing: 2),
        GridItem(.flexible(), spacing: 2),
        GridItem(.flexible(), spacing: 2)
    ]

    private var selectedAssets: [PHAsset] {
        photoLibraryService.assets.filter { selectedAssetIds.contains($0.localIdentifier) }
    }

    var body: some View {
        VStack(spacing: 0) {
            if isLimited {
                HStack {
                    Image(systemName: "info.circle")
                    Text("Limited access - Some photos may not be visible")
                }
                .font(.caption)
                .foregroundColor(.secondary)
                .padding(.horizontal)
                .padding(.vertical, 8)
            }

            if photoLibraryService.isLoading {
                Spacer()
                ProgressView("Loading photos...")
                Spacer()
            } else if photoLibraryService.isEmpty {
                Spacer()
                VStack(spacing: 12) {
                    Image(systemName: "photo.on.rectangle")
                        .font(.system(size: 50))
                        .foregroundColor(.secondary)
                    Text("No Photos")
                        .font(.title2)
                        .foregroundColor(.secondary)
                    Text("Your photo library is empty")
                        .font(.body)
                        .foregroundColor(.secondary)
                }
                Spacer()
            } else {
                ScrollView {
                    LazyVGrid(columns: columns, spacing: 2) {
                        ForEach(Array(photoLibraryService.assets.enumerated()), id: \.element.localIdentifier) { index, asset in
                            ImageThumbnailView(
                                asset: asset,
                                targetSize: CGSize(width: 120, height: 120),
                                isSelectionMode: isSelectionMode,
                                isSelected: selectedAssetIds.contains(asset.localIdentifier)
                            )
                            .onTapGesture {
                                if isSelectionMode {
                                    toggleSelection(for: asset)
                                } else {
                                    selectedPhotoIndex = index
                                    isShowingPhotoDetail = true
                                }
                            }
                            .onLongPressGesture {
                                if !isSelectionMode {
                                    isSelectionMode = true
                                    selectedAssetIds.insert(asset.localIdentifier)
                                }
                            }
                        }
                    }
                }
            }

            // Selection mode toolbar
            if isSelectionMode {
                SelectionToolbar(
                    selectedCount: selectedAssetIds.count,
                    isAddingToAlbum: isAddingToAlbum,
                    onAddToAlbum: {
                        isShowingAlbumPicker = true
                    },
                    onCancel: {
                        exitSelectionMode()
                    }
                )
            }
        }
        .onAppear {
            photoLibraryService.fetchAllAssets()
        }
        .fullScreenCover(isPresented: $isShowingPhotoDetail) {
            PhotoDetailView(
                assets: photoLibraryService.assets,
                selectedIndex: $selectedPhotoIndex
            )
        }
        .sheet(isPresented: $isShowingAlbumPicker) {
            AlbumPickerSheet(
                albumService: albumService,
                isAddingToAlbum: $isAddingToAlbum,
                onSelectAlbum: { album in
                    Task {
                        await addSelectedPhotosToAlbum(album)
                    }
                }
            )
        }
    }

    private func toggleSelection(for asset: PHAsset) {
        if selectedAssetIds.contains(asset.localIdentifier) {
            selectedAssetIds.remove(asset.localIdentifier)
            if selectedAssetIds.isEmpty {
                isSelectionMode = false
            }
        } else {
            selectedAssetIds.insert(asset.localIdentifier)
        }
    }

    private func exitSelectionMode() {
        isSelectionMode = false
        selectedAssetIds.removeAll()
    }

    private func addSelectedPhotosToAlbum(_ album: Album) async {
        isAddingToAlbum = true
        do {
            try await albumService.addAssets(selectedAssets, to: album)
            await MainActor.run {
                isShowingAlbumPicker = false
                exitSelectionMode()
                isAddingToAlbum = false
            }
        } catch {
            await MainActor.run {
                isAddingToAlbum = false
            }
        }
    }
}

struct SelectionToolbar: View {
    let selectedCount: Int
    let isAddingToAlbum: Bool
    let onAddToAlbum: () -> Void
    let onCancel: () -> Void

    var body: some View {
        HStack {
            Button("Cancel") {
                onCancel()
            }
            .foregroundColor(.blue)

            Spacer()

            Text("\(selectedCount) selected")
                .font(.headline)

            Spacer()

            Button {
                onAddToAlbum()
            } label: {
                if isAddingToAlbum {
                    ProgressView()
                        .progressViewStyle(CircularProgressViewStyle())
                } else {
                    Text("Add to Album")
                }
            }
            .disabled(selectedCount == 0 || isAddingToAlbum)
            .foregroundColor(selectedCount == 0 ? .gray : .blue)
        }
        .padding(.horizontal)
        .padding(.vertical, 12)
        .background(Color(UIColor.systemBackground))
        .shadow(color: .black.opacity(0.1), radius: 2, x: 0, y: -1)
    }
}

struct AlbumPickerSheet: View {
    @ObservedObject var albumService: AlbumService
    @Binding var isAddingToAlbum: Bool
    let onSelectAlbum: (Album) -> Void
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Group {
                let userAlbums = albumService.fetchUserAlbums()
                if userAlbums.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "rectangle.stack.badge.plus")
                            .font(.system(size: 50))
                            .foregroundColor(.secondary)
                        Text("No Albums")
                            .font(.title2)
                            .foregroundColor(.secondary)
                        Text("Create an album in the Collections tab first")
                            .font(.body)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                    }
                    .padding()
                } else {
                    List(userAlbums) { album in
                        Button {
                            onSelectAlbum(album)
                        } label: {
                            HStack {
                                if let keyAsset = album.keyAsset {
                                    ImageThumbnailView(
                                        asset: keyAsset,
                                        targetSize: CGSize(width: 60, height: 60)
                                    )
                                    .frame(width: 60, height: 60)
                                    .cornerRadius(4)
                                } else {
                                    Rectangle()
                                        .fill(Color.gray.opacity(0.3))
                                        .frame(width: 60, height: 60)
                                        .cornerRadius(4)
                                }

                                VStack(alignment: .leading) {
                                    Text(album.title)
                                        .foregroundColor(.primary)
                                    Text("\(album.assetCount) items")
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                            }
                        }
                        .disabled(isAddingToAlbum)
                    }
                }
            }
            .navigationTitle("Add to Album")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
        }
    }
}

#Preview {
    GalleryView()
}
