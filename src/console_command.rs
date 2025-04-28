use crate::wav::enhance_image_with_model;
use crate::gaussian_blur::selective_gaussian_blur;
use crate::settings::FunctionsSettings;

use std::cell::RefCell;
use std::rc::Rc;


pub fn generate_images(img_path: &str, function_settings: Rc<RefCell<FunctionsSettings>>) {
    // Call the function to enhance the image with the model
    match enhance_image_with_model(img_path, "model.onnx", 4) {
        Ok(output_path) => println!("Image saved at: {}", output_path),
        Err(e) => eprintln!("Error processing image: {}", e),
    }

    // Call the function to apply selective Gaussian blur
    match selective_gaussian_blur(img_path, &function_settings.borrow()) {
        Ok(output_path) => println!("Image saved at: {}", output_path),
        Err(e) => eprintln!("Error processing image: {}", e),
    }
} 