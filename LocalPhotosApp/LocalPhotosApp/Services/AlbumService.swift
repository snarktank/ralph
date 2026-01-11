//
//  AlbumService.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import Foundation
import Photos

struct Album: Identifiable {
    let id: String
    let collection: PHAssetCollection
    let title: String
    let assetCount: Int
    let keyAsset: PHAsset?

    init(collection: PHAssetCollection) {
        self.id = collection.localIdentifier
        self.collection = collection
        self.title = collection.localizedTitle ?? "Untitled"

        // Fetch assets to get count and key asset
        let fetchOptions = PHFetchOptions()
        fetchOptions.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: false)]
        let assets = PHAsset.fetchAssets(in: collection, options: fetchOptions)

        self.assetCount = assets.count
        self.keyAsset = assets.firstObject
    }
}

@MainActor
class AlbumService: ObservableObject {
    @Published var albums: [Album] = []
    @Published var isLoading: Bool = false

    func fetchAlbums() {
        isLoading = true

        var fetchedAlbums: [Album] = []

        // Fetch Smart Albums (Favorites, Screenshots, etc.)
        let smartAlbumTypes: [PHAssetCollectionSubtype] = [
            .smartAlbumFavorites,
            .smartAlbumScreenshots,
            .smartAlbumSelfPortraits,
            .smartAlbumPanoramas,
            .smartAlbumVideos,
            .smartAlbumRecentlyAdded,
            .smartAlbumBursts,
            .smartAlbumLivePhotos
        ]

        for subtype in smartAlbumTypes {
            let collections = PHAssetCollection.fetchAssetCollections(
                with: .smartAlbum,
                subtype: subtype,
                options: nil
            )

            collections.enumerateObjects { collection, _, _ in
                let album = Album(collection: collection)
                // Only add albums that have at least one asset
                if album.assetCount > 0 {
                    fetchedAlbums.append(album)
                }
            }
        }

        // Fetch User Albums
        let userAlbums = PHAssetCollection.fetchAssetCollections(
            with: .album,
            subtype: .albumRegular,
            options: nil
        )

        userAlbums.enumerateObjects { collection, _, _ in
            let album = Album(collection: collection)
            fetchedAlbums.append(album)
        }

        albums = fetchedAlbums
        isLoading = false
    }

    func fetchAssets(for album: Album) -> [PHAsset] {
        let fetchOptions = PHFetchOptions()
        fetchOptions.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: false)]

        let result = PHAsset.fetchAssets(in: album.collection, options: fetchOptions)

        var assets: [PHAsset] = []
        result.enumerateObjects { asset, _, _ in
            assets.append(asset)
        }

        return assets
    }

    func createAlbum(named title: String) async throws {
        try await PHPhotoLibrary.shared().performChanges {
            PHAssetCollectionChangeRequest.creationRequestForAssetCollection(withTitle: title)
        }
        // Refresh albums list after creation
        fetchAlbums()
    }
}
