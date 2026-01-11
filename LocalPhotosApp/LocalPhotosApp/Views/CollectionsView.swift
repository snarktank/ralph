//
//  CollectionsView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI
import Photos

struct CollectionsView: View {
    @StateObject private var albumService = AlbumService()
    @State private var showNewAlbumAlert = false
    @State private var newAlbumName = ""
    @State private var isCreatingAlbum = false

    var body: some View {
        NavigationStack {
            Group {
                if albumService.isLoading {
                    ProgressView("Loading albums...")
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if albumService.albums.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "rectangle.stack")
                            .font(.system(size: 60))
                            .foregroundColor(.gray)
                        Text("No Albums")
                            .font(.title2)
                            .foregroundColor(.gray)
                        Text("Your albums will appear here")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    List(albumService.albums) { album in
                        NavigationLink(destination: AlbumDetailView(album: album)) {
                            AlbumRowView(album: album)
                        }
                    }
                    .listStyle(.plain)
                }
            }
            .navigationTitle("Collections")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        newAlbumName = ""
                        showNewAlbumAlert = true
                    } label: {
                        Image(systemName: "plus")
                    }
                    .disabled(isCreatingAlbum)
                }
            }
            .alert("New Album", isPresented: $showNewAlbumAlert) {
                TextField("Album name", text: $newAlbumName)
                Button("Cancel", role: .cancel) { }
                Button("Create") {
                    createAlbum()
                }
                .disabled(newAlbumName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            } message: {
                Text("Enter a name for this album.")
            }
            .onAppear {
                if albumService.albums.isEmpty {
                    albumService.fetchAlbums()
                }
            }
        }
    }

    private func createAlbum() {
        let trimmedName = newAlbumName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedName.isEmpty else { return }

        isCreatingAlbum = true
        Task {
            do {
                try await albumService.createAlbum(named: trimmedName)
            } catch {
                print("Failed to create album: \(error)")
            }
            isCreatingAlbum = false
        }
    }
}

struct AlbumRowView: View {
    let album: Album

    private static let imageManager = PHCachingImageManager()
    @State private var thumbnailImage: UIImage?

    var body: some View {
        HStack(spacing: 12) {
            // Album cover thumbnail
            ZStack {
                if let image = thumbnailImage {
                    Image(uiImage: image)
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(width: 70, height: 70)
                        .clipped()
                        .cornerRadius(8)
                } else {
                    Rectangle()
                        .fill(Color.gray.opacity(0.3))
                        .frame(width: 70, height: 70)
                        .cornerRadius(8)
                        .overlay {
                            Image(systemName: "photo")
                                .foregroundColor(.gray)
                        }
                }
            }

            // Album info
            VStack(alignment: .leading, spacing: 4) {
                Text(album.title)
                    .font(.headline)
                    .lineLimit(1)
                Text("\(album.assetCount) \(album.assetCount == 1 ? "item" : "items")")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }

            Spacer()
        }
        .padding(.vertical, 4)
        .onAppear {
            loadThumbnail()
        }
    }

    private func loadThumbnail() {
        guard let keyAsset = album.keyAsset else { return }

        let options = PHImageRequestOptions()
        options.deliveryMode = .opportunistic
        options.isNetworkAccessAllowed = true
        options.resizeMode = .fast

        let scale = UIScreen.main.scale
        let targetSize = CGSize(width: 70 * scale, height: 70 * scale)

        Self.imageManager.requestImage(
            for: keyAsset,
            targetSize: targetSize,
            contentMode: .aspectFill,
            options: options
        ) { result, _ in
            if let result = result {
                DispatchQueue.main.async {
                    self.thumbnailImage = result
                }
            }
        }
    }
}

#Preview {
    CollectionsView()
}
