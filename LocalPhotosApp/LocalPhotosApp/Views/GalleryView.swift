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

    private let columns = [
        GridItem(.flexible(), spacing: 2),
        GridItem(.flexible(), spacing: 2),
        GridItem(.flexible(), spacing: 2)
    ]

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
                        ForEach(photoLibraryService.assets, id: \.localIdentifier) { asset in
                            ImageThumbnailView(
                                asset: asset,
                                targetSize: CGSize(width: 120, height: 120)
                            )
                        }
                    }
                }
            }
        }
        .onAppear {
            photoLibraryService.fetchAllAssets()
        }
    }
}

#Preview {
    GalleryView()
}
