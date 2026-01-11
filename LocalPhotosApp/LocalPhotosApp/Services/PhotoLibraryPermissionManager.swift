//
//  PhotoLibraryPermissionManager.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import Foundation
import Photos

@MainActor
class PhotoLibraryPermissionManager: ObservableObject {
    @Published var authorizationStatus: PHAuthorizationStatus = .notDetermined

    init() {
        authorizationStatus = PHPhotoLibrary.authorizationStatus(for: .readWrite)
    }

    func requestAuthorization() async {
        let status = await PHPhotoLibrary.requestAuthorization(for: .readWrite)
        authorizationStatus = status
    }

    var isAuthorized: Bool {
        authorizationStatus == .authorized || authorizationStatus == .limited
    }

    var statusMessage: String {
        switch authorizationStatus {
        case .notDetermined:
            return "Photo access is required to display your photos."
        case .restricted:
            return "Photo access is restricted. This may be due to parental controls."
        case .denied:
            return "Photo access was denied. Please enable it in Settings to use this app."
        case .authorized:
            return "Full access granted."
        case .limited:
            return "Limited access granted. Some photos may not be visible."
        @unknown default:
            return "Unknown authorization status."
        }
    }
}
