use core::panic;
use std::{cmp::min, ops::Div};

use image::GenericImageView;

type Buffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
type Kernel = Vec<Vec<f64>>;

fn main() {
    let original = image::open("./src/original.jpg").expect("arquivo não existe");
    let img = image_copy(&original);
    let buffer = image_to_buffer(img);
    let buffer = naive_gray_scale(buffer);
    let _ = buffer.save("./src/gray_naive.png");

    let img = image_copy(&original);
    let buffer = image_to_buffer(img);
    let buffer = luminance_gray_scale(buffer);
    let _ = buffer.save("./src/gray_luminance.png");

    let img = image_copy(&original);
    let buffer = image_to_buffer(img);
    let buffer = naive_convolution(buffer, generate_blur_kernel_naive(1., 5));
    let _ = buffer.save("./src/blured.png");

    let img = image_copy(&original);
    let buffer = image_to_buffer(img);
    let buffer = luminance_gray_scale(buffer);
    let buffer = naive_convolution(buffer, generate_blur_kernel_naive(1., 3));
    let edge_detected = naive_edge_detection_sobel(&buffer, &original);

    let _ = edge_detected.save("./src/edge_detection.png");
}

fn generate_blur_kernel_naive(content: f64, size: usize) -> Vec<Vec<f64>> {
    vec![vec![content; size]; size]
}

fn generete_kernel_gaussian() {}

fn image_copy(image: &image::DynamicImage) -> image::DynamicImage {
    image.clone()
}

fn image_to_buffer(image: image::DynamicImage) -> Buffer {
    image.into_rgba8()
}

fn luminance_gray_scale(mut buffer: Buffer) -> Buffer {
    buffer.enumerate_pixels_mut().for_each(|p| {
        let rgba = *p.2;

        let average =
            f32::from(rgba[0]) * 0.299 + f32::from(rgba[1]) * 0.587 + f32::from(rgba[2]) * 0.114;
        let average = average as u8;

        let sub = [average, average, average, 255];

        *p.2 = image::Rgba::from(sub);
    });

    buffer
}

fn naive_gray_scale(mut buffer: Buffer) -> Buffer {
    buffer.enumerate_pixels_mut().for_each(|p| {
        let rgba = *p.2;

        let average = f32::from(rgba[0]) + f32::from(rgba[1]) + f32::from(rgba[2]);
        let average = average.div(3.);
        let average = average as u8;

        let sub = [average, average, average, 255];

        *p.2 = image::Rgba::from(sub);
    });

    buffer
}

fn edge_detection_canny_naive() {}

fn naive_edge_detection_sobel(buffer: &Buffer, original: &image::DynamicImage) -> Buffer {
    let horizontal_kernel = [vec![1., 2., 1.], vec![0., 0., 0.], vec![-1., -2., -1.]];
    let vertical_kernel = [vec![1., 0., -1.], vec![2., 0., -2.], vec![1., 0., -1.]];
    let kernel_dimension = get_dimension(&horizontal_kernel);
    let mut new_buffer = buffer.clone();

    //TODO: dimension check. should be odd

    let padding = 3;
    let kernel_middle = kernel_dimension.width.wrapping_div(2);
    for pixel_y in 0..buffer.height() {
        for pixel_x in 0..buffer.width() {
            let mut horizontal_gradient: f64 = 0.;
            let mut vertical_gradient: f64 = 0.;

            if pixel_y < padding
                || pixel_y >= buffer.height() - padding
                || pixel_x < padding
                || pixel_x >= buffer.width() - padding
            {
                let pixel = new_buffer.get_pixel_mut(pixel_x, pixel_y);
                pixel.0[0] = 0;
                pixel.0[1] = 0;
                pixel.0[2] = 0;

                // let original_pixel = original.get_pixel(pixel_x, pixel_y);
                // pixel.0[0] = original_pixel.0[0];
                // pixel.0[1] = original_pixel.0[1];
                // pixel.0[2] = original_pixel.0[2];

                continue;
            }

            for kernel_y in 0..kernel_dimension.height {
                for kernel_x in 0..kernel_dimension.width {
                    let get_y = pixel_y as usize + kernel_y;
                    let get_x = pixel_x as usize + kernel_x;

                    if get_y < kernel_middle || get_y >= buffer.height() as usize {
                        continue;
                    }

                    if get_x < kernel_middle || get_x >= buffer.width() as usize {
                        continue;
                    }

                    let pixel_value = buffer
                        .get_pixel(
                            (get_x - kernel_middle) as u32,
                            (get_y - kernel_middle) as u32,
                        )
                        .0[0] as f64;

                    let horizontal_kernel_value = horizontal_kernel[kernel_y][kernel_x];
                    let vertical_kernel_value = vertical_kernel[kernel_y][kernel_x];

                    horizontal_gradient += horizontal_kernel_value * pixel_value;
                    vertical_gradient += vertical_kernel_value * pixel_value;
                }
            }

            let total_gradient = (horizontal_gradient.powf(2.) + vertical_gradient.powf(2.)).sqrt();

            let pixel = new_buffer.get_pixel_mut(pixel_x, pixel_y);
            if total_gradient > 90. {
                pixel.0[0] = 234;
                pixel.0[1] = 234;
                pixel.0[2] = 234;
            } else {
                pixel.0[0] = 0;
                pixel.0[1] = 0;
                pixel.0[2] = 0;
            }
        }
    }

    new_buffer
}

fn naive_convolution(mut buffer: Buffer, kernel: Kernel) -> Buffer {
    let kernel_dimension = get_dimension(&kernel);
    let total_conv_pixels = kernel_dimension.height as f64 * kernel_dimension.width as f64;

    //TODO: dimension check. should be odd

    for pixel_y in 0..buffer.height() {
        for pixel_x in 0..buffer.width() {
            let mut conv_result: Vec<f64> = vec![0., 0., 0.];
            for kernel_row in kernel.iter().enumerate() {
                for kernel_column in kernel_row.1.iter().enumerate() {
                    let kernel_y = kernel_row.0;
                    let kernel_x = kernel_column.0;

                    let half_point = kernel_dimension.width.wrapping_div(2);
                    let get_y: i64 = pixel_y as i64 + kernel_y as i64;
                    let get_x: i64 = pixel_x as i64 + kernel_x as i64;

                    if get_y < half_point as i64 || get_y >= buffer.height().into() {
                        continue;
                    }

                    if get_x < half_point as i64 || get_x >= buffer.width().into() {
                        continue;
                    }

                    let pixel_rgba = buffer
                        .get_pixel(
                            (get_x - half_point as i64) as u32,
                            (get_y - half_point as i64) as u32,
                        )
                        .0;

                    let kernel_mult = kernel_column.1;
                    conv_result[0] += kernel_mult * pixel_rgba[0] as f64;
                    conv_result[1] += kernel_mult * pixel_rgba[1] as f64;
                    conv_result[2] += kernel_mult * pixel_rgba[2] as f64;
                }
            }

            let pixel = buffer.get_pixel_mut(pixel_x, pixel_y);
            pixel.0[0] = min((conv_result[0] / total_conv_pixels) as i64, 254) as u8;
            pixel.0[1] = min((conv_result[1] / total_conv_pixels) as i64, 254) as u8;
            pixel.0[2] = min((conv_result[2] / total_conv_pixels) as i64, 254) as u8;
        }
    }

    buffer
}

#[derive(Copy, Clone)]
struct Dimension {
    height: usize,
    width: usize,
}
fn get_dimension<T>(kernel: &[Vec<T>]) -> Dimension
where
    T: Sized,
{
    //todo: implement errors. Can't get dimensions of uneven Vecs
    let rows_quantity = kernel.len();
    let columns_quantity = kernel[0].len();

    Dimension {
        height: columns_quantity,
        width: rows_quantity,
    }
}
