//
//  PlacesView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI
import MapKit
import Photos

struct PlacesView: View {
    @StateObject private var locationService = LocationService()
    @State private var selectedLocation: PhotoLocation?
    @State private var showingLocationPhotos = false
    @State private var cameraPosition: MapCameraPosition = .automatic
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ZStack {
                if locationService.isLoading {
                    ProgressView("Loading locations...")
                } else if !locationService.hasLocations {
                    VStack(spacing: 16) {
                        Image(systemName: "location.slash")
                            .font(.system(size: 60))
                            .foregroundColor(.gray)
                        Text("No Photo Locations")
                            .font(.title2)
                            .foregroundColor(.gray)
                        Text("Photos without location data won't appear on the map")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal)
                    }
                } else {
                    Map(position: $cameraPosition) {
                        ForEach(locationService.photoLocations) { location in
                            Annotation(
                                "\(location.assetCount)",
                                coordinate: location.coordinate
                            ) {
                                PhotoLocationPin(
                                    photoLocation: location,
                                    onTap: {
                                        selectedLocation = location
                                        showingLocationPhotos = true
                                    }
                                )
                            }
                        }
                    }
                    .mapStyle(.standard)
                }
            }
            .navigationTitle("Places")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                if locationService.photoLocations.isEmpty {
                    locationService.fetchPhotosWithLocations()
                }
            }
            .sheet(isPresented: $showingLocationPhotos) {
                if let location = selectedLocation {
                    LocationPhotosView(photoLocation: location)
                }
            }
        }
    }
}

// MARK: - Photo Location Pin

struct PhotoLocationPin: View {
    let photoLocation: PhotoLocation
    let onTap: () -> Void

    private static let imageManager = PHCachingImageManager()
    @State private var thumbnailImage: UIImage?

    var body: some View {
        Button(action: onTap) {
            VStack(spacing: 0) {
                ZStack {
                    // Thumbnail circle
                    if let image = thumbnailImage {
                        Image(uiImage: image)
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                            .frame(width: 44, height: 44)
                            .clipShape(Circle())
                            .overlay(
                                Circle()
                                    .stroke(Color.white, lineWidth: 2)
                            )
                    } else {
                        Circle()
                            .fill(Color.blue)
                            .frame(width: 44, height: 44)
                            .overlay(
                                Image(systemName: "photo")
                                    .foregroundColor(.white)
                            )
                            .overlay(
                                Circle()
                                    .stroke(Color.white, lineWidth: 2)
                            )
                    }

                    // Badge showing photo count
                    if photoLocation.assetCount > 1 {
                        Text("\(photoLocation.assetCount)")
                            .font(.caption2)
                            .fontWeight(.bold)
                            .foregroundColor(.white)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.blue)
                            .clipShape(Capsule())
                            .offset(x: 18, y: -18)
                    }
                }

                // Pin pointer
                Image(systemName: "arrowtriangle.down.fill")
                    .font(.system(size: 10))
                    .foregroundColor(.white)
                    .offset(y: -4)
            }
            .shadow(radius: 3)
        }
        .buttonStyle(.plain)
        .onAppear {
            loadThumbnail()
        }
    }

    private func loadThumbnail() {
        guard let keyAsset = photoLocation.keyAsset else { return }

        let options = PHImageRequestOptions()
        options.deliveryMode = .opportunistic
        options.isNetworkAccessAllowed = true
        options.resizeMode = .fast

        let scale = UIScreen.main.scale
        let targetSize = CGSize(width: 44 * scale, height: 44 * scale)

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

// MARK: - Location Photos View (Sheet)

struct LocationPhotosView: View {
    let photoLocation: PhotoLocation
    @Environment(\.dismiss) private var dismiss

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
        NavigationStack {
            ScrollView {
                if assets.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "photo")
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
            .navigationTitle("\(photoLocation.assetCount) Photos")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                assets = photoLocation.assets
            }
            .fullScreenCover(isPresented: $showingPhotoDetail) {
                if selectedAsset != nil {
                    PhotoDetailView(
                        assets: assets,
                        initialIndex: selectedIndex
                    )
                }
            }
        }
    }
}

#Preview {
    PlacesView()
}
