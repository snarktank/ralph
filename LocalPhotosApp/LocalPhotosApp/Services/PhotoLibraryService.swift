//
//  PhotoLibraryService.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import Foundation
import Photos

@MainActor
class PhotoLibraryService: ObservableObject {
    @Published var assets: [PHAsset] = []
    @Published var isLoading: Bool = false
    @Published var isEmpty: Bool = true

    private var fetchResult: PHFetchResult<PHAsset>?

    func fetchAllAssets() {
        isLoading = true

        let fetchOptions = PHFetchOptions()
        fetchOptions.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: false)]

        // Fetch both images and videos
        fetchOptions.predicate = NSPredicate(
            format: "mediaType == %d OR mediaType == %d",
            PHAssetMediaType.image.rawValue,
            PHAssetMediaType.video.rawValue
        )

        fetchResult = PHAsset.fetchAssets(with: fetchOptions)

        guard let result = fetchResult else {
            assets = []
            isEmpty = true
            isLoading = false
            return
        }

        var fetchedAssets: [PHAsset] = []
        result.enumerateObjects { asset, _, _ in
            fetchedAssets.append(asset)
        }

        assets = fetchedAssets
        isEmpty = fetchedAssets.isEmpty
        isLoading = false
    }

    func fetchImages() -> [PHAsset] {
        let fetchOptions = PHFetchOptions()
        fetchOptions.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: false)]
        fetchOptions.predicate = NSPredicate(format: "mediaType == %d", PHAssetMediaType.image.rawValue)

        let result = PHAsset.fetchAssets(with: fetchOptions)

        var fetchedAssets: [PHAsset] = []
        result.enumerateObjects { asset, _, _ in
            fetchedAssets.append(asset)
        }

        return fetchedAssets
    }

    func fetchVideos() -> [PHAsset] {
        let fetchOptions = PHFetchOptions()
        fetchOptions.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: false)]
        fetchOptions.predicate = NSPredicate(format: "mediaType == %d", PHAssetMediaType.video.rawValue)

        let result = PHAsset.fetchAssets(with: fetchOptions)

        var fetchedAssets: [PHAsset] = []
        result.enumerateObjects { asset, _, _ in
            fetchedAssets.append(asset)
        }

        return fetchedAssets
    }

    var assetCount: Int {
        assets.count
    }

    var imageCount: Int {
        assets.filter { $0.mediaType == .image }.count
    }

    var videoCount: Int {
        assets.filter { $0.mediaType == .video }.count
    }
}
