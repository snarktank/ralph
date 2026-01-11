//
//  ImageThumbnailView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI
import Photos

struct ImageThumbnailView: View {
    let asset: PHAsset
    let targetSize: CGSize

    @State private var image: UIImage?

    private static let imageManager = PHCachingImageManager()

    private var isVideo: Bool {
        asset.mediaType == .video
    }

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                if let image = image {
                    Image(uiImage: image)
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(width: geometry.size.width, height: geometry.size.height)
                        .clipped()
                } else {
                    Rectangle()
                        .fill(Color.gray.opacity(0.3))
                }

                // Play icon overlay for videos
                if isVideo {
                    Image(systemName: "play.circle.fill")
                        .font(.system(size: 36))
                        .foregroundStyle(.white, .black.opacity(0.5))
                        .shadow(radius: 2)
                }
            }
        }
        .aspectRatio(1, contentMode: .fit)
        .onAppear {
            loadThumbnail()
        }
    }

    private func loadThumbnail() {
        let options = PHImageRequestOptions()
        options.deliveryMode = .opportunistic
        options.isNetworkAccessAllowed = true
        options.resizeMode = .fast

        let scale = UIScreen.main.scale
        let scaledSize = CGSize(width: targetSize.width * scale, height: targetSize.height * scale)

        Self.imageManager.requestImage(
            for: asset,
            targetSize: scaledSize,
            contentMode: .aspectFill,
            options: options
        ) { result, info in
            if let result = result {
                DispatchQueue.main.async {
                    self.image = result
                }
            }
        }
    }
}

#Preview {
    ImageThumbnailView(
        asset: PHAsset(),
        targetSize: CGSize(width: 120, height: 120)
    )
    .frame(width: 120, height: 120)
}
