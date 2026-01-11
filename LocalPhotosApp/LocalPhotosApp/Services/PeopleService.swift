//
//  PeopleService.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import Foundation
import Photos

struct Person: Identifiable {
    let id: String
    let collection: PHAssetCollection
    let name: String
    let assetCount: Int
    let keyAsset: PHAsset?

    init(collection: PHAssetCollection) {
        self.id = collection.localIdentifier
        self.collection = collection
        self.name = collection.localizedTitle ?? "Unknown"

        // Fetch assets to get count and key asset (face thumbnail)
        let fetchOptions = PHFetchOptions()
        fetchOptions.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: false)]
        let assets = PHAsset.fetchAssets(in: collection, options: fetchOptions)

        self.assetCount = assets.count
        self.keyAsset = assets.firstObject
    }
}

@MainActor
class PeopleService: ObservableObject {
    @Published var people: [Person] = []
    @Published var isLoading: Bool = false

    func fetchPeople() {
        isLoading = true

        var fetchedPeople: [Person] = []

        // Try to fetch People smart album (faces grouped by iOS)
        // Note: iOS groups faces automatically, but access may be limited
        // We'll use the smart album for faces if available

        // Method 1: Fetch face collections from smart albums
        // iOS stores People as a special collection type
        let faceCollections = PHCollectionList.fetchCollectionLists(
            with: .smartFolder,
            subtype: .smartFolderFaces,
            options: nil
        )

        faceCollections.enumerateObjects { collectionList, _, _ in
            // Enumerate collections within the Faces folder
            let collections = PHCollection.fetchCollections(
                in: collectionList,
                options: nil
            )

            collections.enumerateObjects { collection, _, _ in
                if let assetCollection = collection as? PHAssetCollection {
                    let person = Person(collection: assetCollection)
                    if person.assetCount > 0 {
                        fetchedPeople.append(person)
                    }
                }
            }
        }

        // Method 2: If no faces found via folder, try direct smart album fetch
        // (This approach works on some iOS versions)
        if fetchedPeople.isEmpty {
            // Try fetching all smart albums and look for face-related ones
            let allSmartAlbums = PHAssetCollection.fetchAssetCollections(
                with: .smartAlbum,
                subtype: .any,
                options: nil
            )

            allSmartAlbums.enumerateObjects { collection, _, _ in
                // Check if this might be a face-related album by checking title patterns
                // Note: System face albums have specific characteristics
                let title = collection.localizedTitle ?? ""

                // Skip known non-face smart albums
                let nonFaceAlbums = [
                    "Favorites", "Screenshots", "Selfies", "Panoramas",
                    "Videos", "Recently Added", "Bursts", "Live Photos",
                    "Hidden", "Recently Deleted", "Portrait", "Time-lapse",
                    "Slo-mo", "Animated", "Long Exposure", "Unable to Upload",
                    "RAW", "Recents", "Camera Roll", "All Photos"
                ]

                if !nonFaceAlbums.contains(where: { title.localizedCaseInsensitiveContains($0) }) {
                    // This could be a face album - check if it has assets
                    let fetchOptions = PHFetchOptions()
                    let assets = PHAsset.fetchAssets(in: collection, options: fetchOptions)

                    // Face albums typically have multiple photos of the same person
                    if assets.count > 0 {
                        // Additional heuristic: face albums usually don't have generic names
                        // and tend to have person names
                        let person = Person(collection: collection)
                        // Only add if not already in our list (by ID)
                        if !fetchedPeople.contains(where: { $0.id == person.id }) {
                            fetchedPeople.append(person)
                        }
                    }
                }
            }
        }

        // Sort by name
        fetchedPeople.sort { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }

        people = fetchedPeople
        isLoading = false
    }

    func fetchAssets(for person: Person) -> [PHAsset] {
        let fetchOptions = PHFetchOptions()
        fetchOptions.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: false)]

        let result = PHAsset.fetchAssets(in: person.collection, options: fetchOptions)

        var assets: [PHAsset] = []
        result.enumerateObjects { asset, _, _ in
            assets.append(asset)
        }

        return assets
    }
}
