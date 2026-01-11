//
//  CreateView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI

struct FeatureCard: Identifiable {
    let id = UUID()
    let title: String
    let description: String
    let iconName: String
    let isComingSoon: Bool
}

struct FeatureCardView: View {
    let feature: FeatureCard
    let onTap: (() -> Void)?

    var body: some View {
        Button(action: {
            if !feature.isComingSoon {
                onTap?()
            }
        }) {
            VStack(spacing: 12) {
                Image(systemName: feature.iconName)
                    .font(.system(size: 40))
                    .foregroundColor(feature.isComingSoon ? .gray : .blue)

                Text(feature.title)
                    .font(.headline)
                    .foregroundColor(feature.isComingSoon ? .gray : .primary)

                Text(feature.description)
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)
                    .lineLimit(2)
            }
            .frame(maxWidth: .infinity)
            .padding()
            .background(Color(.systemGray6))
            .cornerRadius(16)
            .overlay(alignment: .topTrailing) {
                if feature.isComingSoon {
                    Text("Coming Soon")
                        .font(.caption2)
                        .fontWeight(.semibold)
                        .foregroundColor(.white)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(Color.orange)
                        .cornerRadius(8)
                        .padding(8)
                }
            }
        }
        .buttonStyle(.plain)
        .disabled(feature.isComingSoon)
    }
}

struct CreateView: View {
    @State private var showCollagePicker = false

    private let features: [FeatureCard] = [
        FeatureCard(
            title: "Collage",
            description: "Combine multiple photos into one creative layout",
            iconName: "square.grid.2x2.fill",
            isComingSoon: false
        ),
        FeatureCard(
            title: "Movie",
            description: "Create slideshow movies from your photos",
            iconName: "film.fill",
            isComingSoon: true
        ),
        FeatureCard(
            title: "Animation",
            description: "Turn your photos into animated GIFs",
            iconName: "sparkles.rectangle.stack.fill",
            isComingSoon: true
        ),
        FeatureCard(
            title: "Highlight Reel",
            description: "Auto-generate video highlights from your best moments",
            iconName: "star.circle.fill",
            isComingSoon: true
        )
    ]

    private let columns = [
        GridItem(.flexible(), spacing: 16),
        GridItem(.flexible(), spacing: 16)
    ]

    var body: some View {
        NavigationStack {
            ScrollView {
                LazyVGrid(columns: columns, spacing: 16) {
                    ForEach(features) { feature in
                        FeatureCardView(feature: feature) {
                            if feature.title == "Collage" {
                                showCollagePicker = true
                            }
                        }
                    }
                }
                .padding()
            }
            .navigationTitle("Create")
            .sheet(isPresented: $showCollagePicker) {
                CollagePickerView()
            }
        }
    }
}

#Preview {
    CreateView()
}
