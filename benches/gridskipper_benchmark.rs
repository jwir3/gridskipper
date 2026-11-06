use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::DynamicImage;
// use gridskipper::{compare_images_pixel_by_pixel, DiffOptions};
use gridskipper::{diff::{DiffOptions, compare_images_pixel_by_pixel}, utils::generate_random_image};

fn benchmark_create_large_image(c: &mut Criterion) {
    // Setup: load or generate your test images once before the timer starts
    // let img1 = DynamicImage::new_rgba8(100, 100); 
    // let img2 = DynamicImage::new_rgba8(100, 100);
    // let options = DiffOptions { threshold: 0.0 }; // Mock options
    let diff_image_1 = generate_random_image(1024 * 16);
    let diff_image_2 = generate_random_image(1024 * 16);

    c.bench_function("compare_images_pixel_by_pixel_gridskipper", |b| {
        b.iter(|| {
            // black_box prevents the compiler from optimizing away the function call
            // compare_images_pixel_by_pixel(
            //     black_box(img1.clone()), 
            //     black_box(img2.clone()), 
            //     black_box(&options)
            // generate_random_image(1024 * 32);
            // ).unwrap();
            compare_images_pixel_by_pixel(
                black_box(DynamicImage::from(diff_image_1.clone())), 
                black_box(DynamicImage::from(diff_image_2.clone())), 
                black_box(&DiffOptions::default())
            ).unwrap();
        })
    });
}

criterion_group!(benches, benchmark_create_large_image);
criterion_main!(benches);