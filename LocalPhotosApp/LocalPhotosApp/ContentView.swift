//
//  ContentView.swift
//  LocalPhotosApp
//
//  Created by Ralph
//

import SwiftUI

struct ContentView: View {
    var body: some View {
        TabView {
            GalleryView()
                .tabItem {
                    Label("Gallery", systemImage: "photo.on.rectangle")
                }

            CollectionsView()
                .tabItem {
                    Label("Collections", systemImage: "rectangle.stack")
                }

            CreateView()
                .tabItem {
                    Label("Create", systemImage: "plus.circle")
                }
        }
    }
}

#Preview {
    ContentView()
}
