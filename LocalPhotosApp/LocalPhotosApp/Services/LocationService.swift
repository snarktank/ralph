//
//  LocationService.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import Foundation
import Photos
import CoreLocation

struct PhotoLocation: Identifiable {
    let id: String
    let coordinate: CLLocationCoordinate2D
    let assets: [PHAsset]

    var assetCount: Int { assets.count }
    var keyAsset: PHAsset? { assets.first }
}

@MainActor
class LocationService: ObservableObject {
    @Published var photoLocations: [PhotoLocation] = []
    @Published var isLoading: Bool = false
    @Published var hasLocations: Bool = false

    // Clustering threshold in degrees (roughly ~100m at equator)
    private let clusterThreshold: Double = 0.001

    func fetchPhotosWithLocations() {
        isLoading = true

        // Fetch all assets with location data
        let fetchOptions = PHFetchOptions()
        fetchOptions.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: false)]

        let allAssets = PHAsset.fetchAssets(with: fetchOptions)

        var assetsWithLocation: [(asset: PHAsset, location: CLLocation)] = []

        allAssets.enumerateObjects { asset, _, _ in
            if let location = asset.location {
                assetsWithLocation.append((asset, location))
            }
        }

        // Group assets by location (simple clustering)
        let groupedLocations = clusterAssetsByLocation(assetsWithLocation)

        photoLocations = groupedLocations
        hasLocations = !photoLocations.isEmpty
        isLoading = false
    }

    private func clusterAssetsByLocation(_ assetsWithLocation: [(asset: PHAsset, location: CLLocation)]) -> [PhotoLocation] {
        var clusters: [[PHAsset]] = []
        var clusterCenters: [CLLocationCoordinate2D] = []

        for (asset, location) in assetsWithLocation {
            let coord = location.coordinate

            // Find if this location belongs to an existing cluster
            var foundCluster = false
            for (index, center) in clusterCenters.enumerated() {
                if isNearby(coord, center) {
                    clusters[index].append(asset)
                    foundCluster = true
                    break
                }
            }

            // Create new cluster if no nearby cluster found
            if !foundCluster {
                clusters.append([asset])
                clusterCenters.append(coord)
            }
        }

        // Convert clusters to PhotoLocation objects
        var photoLocations: [PhotoLocation] = []
        for (index, assets) in clusters.enumerated() {
            let center = clusterCenters[index]
            let photoLocation = PhotoLocation(
                id: "\(center.latitude)_\(center.longitude)",
                coordinate: center,
                assets: assets
            )
            photoLocations.append(photoLocation)
        }

        // Sort by asset count (most photos first)
        photoLocations.sort { $0.assetCount > $1.assetCount }

        return photoLocations
    }

    private func isNearby(_ coord1: CLLocationCoordinate2D, _ coord2: CLLocationCoordinate2D) -> Bool {
        let latDiff = abs(coord1.latitude - coord2.latitude)
        let lonDiff = abs(coord1.longitude - coord2.longitude)
        return latDiff < clusterThreshold && lonDiff < clusterThreshold
    }

    func fetchAssets(for photoLocation: PhotoLocation) -> [PHAsset] {
        return photoLocation.assets
    }
}
