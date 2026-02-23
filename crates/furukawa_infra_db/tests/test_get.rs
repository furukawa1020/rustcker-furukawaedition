use furukawa_infra_db::sqlite::SqliteStore;
use furukawa_domain::image::store::ImageMetadataStore;
use std::env;

#[tokio::test]
async fn debug_get_image() {
    let db_path = "sqlite://C:/Projects/furukawa_data_demo/furukawa.db?mode=rwc";
    println!("Connecting to DB: {}", db_path);
    let store = SqliteStore::new(db_path).await.expect("Failed to init store");

    println!("Listing all images...");
    let all = store.list().await.unwrap();
    for img in all {
        println!(" - ID: {}, Tags: {:?}", img.id, img.repo_tags);
    }

    let search1 = "alpine:latest";
    println!("Getting by tag '{}'...", search1);
    match store.get(search1).await.unwrap() {
        Some(img) => println!("Found: {:?}", img),
        None => println!("NOT FOUND: {}", search1),
    }

    let search2 = "library/alpine:latest";
    println!("Getting by tag '{}'...", search2);
    match store.get(search2).await.unwrap() {
        Some(img) => println!("Found: {:?}", img),
        None => println!("NOT FOUND: {}", search2),
    }

    let search3 = "sha256:a40c03cbb81c59bfb0e0887ab0b1859727075da7b9cc576a1cec2c771f38c5fb";
    println!("Getting by ID '{}'...", search3);
    match store.get(search3).await.unwrap() {
        Some(img) => println!("Found: {:?}", img),
        None => println!("NOT FOUND: {}", search3),
    }
}
