//
//  AlbumDetailView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI
import Photos

struct AlbumDetailView: View {
    let album: Album
    @StateObject private var albumService = AlbumService()

    @State private var assets: [PHAsset] = []
    @State private var selectedAsset: PHAsset?
    @State private var selectedIndex: Int = 0
    @State private var showingPhotoDetail = false

    private let columns = [
        GridItem(.flexible(), spacing: 2),
        GridItem(.flexible(), spacing: 2),
        GridItem(.flexible(), spacing: 2)
    ]

    var body: some View {
        ScrollView {
            if assets.isEmpty {
                VStack(spacing: 16) {
                    Image(systemName: "photo.on.rectangle.angled")
                        .font(.system(size: 60))
                        .foregroundColor(.gray)
                    Text("No Photos")
                        .font(.title2)
                        .foregroundColor(.gray)
                }
                .frame(maxWidth: .infinity, minHeight: 300)
            } else {
                LazyVGrid(columns: columns, spacing: 2) {
                    ForEach(Array(assets.enumerated()), id: \.element.localIdentifier) { index, asset in
                        ImageThumbnailView(
                            asset: asset,
                            targetSize: CGSize(width: 120, height: 120)
                        )
                        .onTapGesture {
                            selectedAsset = asset
                            selectedIndex = index
                            showingPhotoDetail = true
                        }
                    }
                }
                .padding(.horizontal, 2)
            }
        }
        .navigationTitle(album.title)
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            assets = albumService.fetchAssets(for: album)
        }
        .fullScreenCover(isPresented: $showingPhotoDetail) {
            if selectedAsset != nil {
                PhotoDetailView(
                    assets: assets,
                    selectedIndex: $selectedIndex
                )
            }
        }
    }
}

#Preview {
    NavigationStack {
        Text("Album Detail Preview")
    }
}
