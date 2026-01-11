//
//  VideoPlayerView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI
import AVKit
import Photos

struct VideoPlayerView: View {
    let asset: PHAsset

    @State private var player: AVPlayer?
    @State private var isLoading = true

    var body: some View {
        ZStack {
            if let player = player {
                VideoPlayer(player: player)
                    .onAppear {
                        player.play()
                    }
                    .onDisappear {
                        player.pause()
                    }
            } else if isLoading {
                ProgressView()
                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                    .scaleEffect(1.5)
            } else {
                VStack {
                    Image(systemName: "exclamationmark.triangle")
                        .font(.system(size: 40))
                        .foregroundColor(.white.opacity(0.7))
                    Text("Unable to load video")
                        .foregroundColor(.white.opacity(0.7))
                }
            }
        }
        .onAppear {
            loadVideo()
        }
    }

    private func loadVideo() {
        let options = PHVideoRequestOptions()
        options.isNetworkAccessAllowed = true
        options.deliveryMode = .automatic

        PHImageManager.default().requestPlayerItem(forVideo: asset, options: options) { playerItem, info in
            DispatchQueue.main.async {
                if let playerItem = playerItem {
                    self.player = AVPlayer(playerItem: playerItem)
                }
                self.isLoading = false
            }
        }
    }
}

#Preview {
    VideoPlayerView(asset: PHAsset())
        .background(Color.black)
}
