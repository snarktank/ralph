//
//  CollectionsView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI
import Photos
import MapKit

struct CollectionsView: View {
    @StateObject private var albumService = AlbumService()
    @StateObject private var peopleService = PeopleService()
    @StateObject private var locationService = LocationService()
    @State private var showNewAlbumAlert = false
    @State private var newAlbumName = ""
    @State private var isCreatingAlbum = false
    @State private var showPlacesView = false

    var body: some View {
        NavigationStack {
            Group {
                if albumService.isLoading && peopleService.isLoading && locationService.isLoading {
                    ProgressView("Loading...")
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    ScrollView {
                        VStack(alignment: .leading, spacing: 24) {
                            // People Section
                            PeopleSectionView(peopleService: peopleService)

                            // Places Section
                            PlacesSectionView(
                                locationService: locationService,
                                onTap: { showPlacesView = true }
                            )

                            // Albums Section
                            AlbumsSectionView(albumService: albumService)
                        }
                        .padding(.vertical)
                    }
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
                if peopleService.people.isEmpty {
                    peopleService.fetchPeople()
                }
                if locationService.photoLocations.isEmpty {
                    locationService.fetchPhotosWithLocations()
                }
            }
            .fullScreenCover(isPresented: $showPlacesView) {
                PlacesView()
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

// MARK: - People Section

struct PeopleSectionView: View {
    @ObservedObject var peopleService: PeopleService

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Section Header
            Text("People")
                .font(.title2)
                .fontWeight(.bold)
                .padding(.horizontal)

            if peopleService.isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()
            } else if peopleService.people.isEmpty {
                // Placeholder when no people found
                HStack {
                    Spacer()
                    VStack(spacing: 8) {
                        Image(systemName: "person.crop.circle.badge.questionmark")
                            .font(.system(size: 40))
                            .foregroundColor(.gray)
                        Text("No people found")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .padding(.vertical, 20)
                    Spacer()
                }
            } else {
                // Horizontal scroll row of detected faces
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 16) {
                        ForEach(peopleService.people) { person in
                            NavigationLink(destination: PersonDetailView(person: person)) {
                                PersonCircleView(person: person)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .padding(.horizontal)
                }
            }
        }
    }
}

struct PersonCircleView: View {
    let person: Person

    private static let imageManager = PHCachingImageManager()
    @State private var thumbnailImage: UIImage?

    var body: some View {
        VStack(spacing: 6) {
            // Circular face thumbnail
            ZStack {
                if let image = thumbnailImage {
                    Image(uiImage: image)
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(width: 70, height: 70)
                        .clipShape(Circle())
                } else {
                    Circle()
                        .fill(Color.gray.opacity(0.3))
                        .frame(width: 70, height: 70)
                        .overlay {
                            Image(systemName: "person.fill")
                                .font(.system(size: 30))
                                .foregroundColor(.gray)
                        }
                }
            }

            // Person name
            Text(person.name)
                .font(.caption)
                .lineLimit(1)
                .frame(width: 70)
        }
        .onAppear {
            loadThumbnail()
        }
    }

    private func loadThumbnail() {
        guard let keyAsset = person.keyAsset else { return }

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

// MARK: - Places Section

struct PlacesSectionView: View {
    @ObservedObject var locationService: LocationService
    let onTap: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Section Header
            Text("Places")
                .font(.title2)
                .fontWeight(.bold)
                .padding(.horizontal)

            if locationService.isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()
            } else {
                // Map thumbnail preview - always show, tap to open full map
                Button(action: onTap) {
                    PlacesMapPreview(locationService: locationService)
                }
                .buttonStyle(.plain)
                .padding(.horizontal)
            }
        }
    }
}

struct PlacesMapPreview: View {
    @ObservedObject var locationService: LocationService
    @State private var cameraPosition: MapCameraPosition = .automatic

    var body: some View {
        ZStack(alignment: .bottomLeading) {
            if locationService.hasLocations {
                // Mini map with pins
                Map(position: $cameraPosition, interactionModes: []) {
                    ForEach(locationService.photoLocations) { location in
                        Marker("", coordinate: location.coordinate)
                            .tint(.red)
                    }
                }
                .mapStyle(.standard)
                .frame(height: 180)
                .cornerRadius(12)
                .allowsHitTesting(false)
            } else {
                // Placeholder map when no locations
                ZStack {
                    Rectangle()
                        .fill(Color.gray.opacity(0.2))
                        .frame(height: 180)
                        .cornerRadius(12)

                    VStack(spacing: 8) {
                        Image(systemName: "map")
                            .font(.system(size: 40))
                            .foregroundColor(.gray)
                        Text("No locations found")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                }
            }

            // Location count badge
            if locationService.hasLocations {
                HStack(spacing: 4) {
                    Image(systemName: "mappin.circle.fill")
                        .foregroundColor(.red)
                    Text("\(locationService.photoLocations.count) \(locationService.photoLocations.count == 1 ? "location" : "locations")")
                        .font(.caption)
                        .fontWeight(.medium)
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(.ultraThinMaterial)
                .cornerRadius(16)
                .padding(8)
            }

            // "View All" indicator
            HStack {
                Spacer()
                HStack(spacing: 4) {
                    Text("View Map")
                        .font(.caption)
                        .fontWeight(.medium)
                    Image(systemName: "chevron.right")
                        .font(.caption)
                }
                .foregroundColor(.blue)
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(.ultraThinMaterial)
                .cornerRadius(16)
                .padding(8)
            }
        }
    }
}

// MARK: - Albums Section

struct AlbumsSectionView: View {
    @ObservedObject var albumService: AlbumService

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Section Header
            Text("Albums")
                .font(.title2)
                .fontWeight(.bold)
                .padding(.horizontal)

            if albumService.albums.isEmpty {
                HStack {
                    Spacer()
                    VStack(spacing: 8) {
                        Image(systemName: "rectangle.stack")
                            .font(.system(size: 40))
                            .foregroundColor(.gray)
                        Text("No albums")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .padding(.vertical, 20)
                    Spacer()
                }
            } else {
                VStack(spacing: 0) {
                    ForEach(albumService.albums) { album in
                        NavigationLink(destination: AlbumDetailView(album: album)) {
                            AlbumRowView(album: album)
                        }
                        .buttonStyle(.plain)
                        Divider()
                            .padding(.leading, 82)
                    }
                }
                .padding(.horizontal)
            }
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

            Image(systemName: "chevron.right")
                .font(.system(size: 14, weight: .semibold))
                .foregroundColor(.gray.opacity(0.5))
        }
        .padding(.vertical, 8)
        .contentShape(Rectangle())
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
