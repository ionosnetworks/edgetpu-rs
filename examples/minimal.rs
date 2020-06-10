use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bytes::Bytes;
use edgetpu::EdgeTpuContext;
use image;
use tflite::op_resolver::OpResolver;
use tflite::ops::builtin::BuiltinOpResolver;
use tflite::{FlatBufferModel, InterpreterBuilder};

struct Image {
    width: usize,
    height: usize,
    channels: usize,
    data: Bytes,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
}

fn read_bmp<T: AsRef<Path>>(input_bmp_name: T) -> Result<Image, image::ImageError> {
    let (width, height) = image::image_dimensions(input_bmp_name.as_ref())?;
    let im = image::open(input_bmp_name.as_ref())?;
    let channels = im.color().channel_count();

    Ok(Image {
        width: width as usize,
        height: height as usize,
        channels: channels as usize,
        data: Bytes::from(im.to_bytes()),
    })
}

pub fn main() {
    let im = read_bmp(manifest_dir().join("data/resized_cat.bmp")).expect("faild to load image");
    let model = FlatBufferModel::build_from_file(
        manifest_dir().join("data/mobilenet_v1_1.0_224_quant_edgetpu.tflite"),
    )
    .expect("failed to load model");

    let edgetpu_context = EdgeTpuContext::open_device().expect("failed to open coral device");
    let resolver = BuiltinOpResolver::default();
    resolver.add_custom(edgetpu::custom_op(), edgetpu::register_custom_op());

    let builder =
        InterpreterBuilder::new(model, &resolver).expect("must create interpreter builder");
    let mut interpreter = builder.build().expect("must build interpreter");

    interpreter.set_external_context(
        tflite::ExternalContextType::EdgeTpu,
        edgetpu_context.to_external_context(),
    );
    interpreter.set_num_threads(1);
    interpreter
        .allocate_tensors()
        .expect("failed to allocate tensors.");

    let tensor_index = interpreter.inputs()[0];
    let required_shape = interpreter.tensor_info(tensor_index).unwrap().dims;
    if im.height != required_shape[1]
        || im.width != required_shape[2]
        || im.channels != required_shape[3]
    {
        eprintln!("Input size mismatches:");
        eprintln!("\twidth: {} vs {}", im.width, required_shape[0]);
        eprintln!("\theight: {} vs {}", im.height, required_shape[1]);
        eprintln!("\tchannels: {} vs {}", im.channels, required_shape[2]);
        return;
    }

    let inf_start = Instant::now();
    interpreter
        .tensor_data_mut(tensor_index)
        .unwrap()
        .copy_from_slice(im.data.as_ref());
    interpreter.invoke().expect("invoke failed");
    let outputs = interpreter.outputs();
    let mut results = Vec::new();
    for &output in outputs {
        let tensor_info = interpreter.tensor_info(output).expect("must data");
        match tensor_info.element_kind {
            tflite::context::ElementKind::kTfLiteUInt8 => {
                let out_tensor: &[u8] = interpreter.tensor_data(output).expect("must data");
                let scale = tensor_info.params.scale;
                let zero_point = tensor_info.params.zero_point;
                results = out_tensor
                    .into_iter()
                    .map(|&x| scale * (((x as i32) - zero_point) as f32))
                    .collect();
            }
            tflite::context::ElementKind::kTfLiteFloat32 => {
                let out_tensor: &[f32] = interpreter.tensor_data(output).expect("must data");
                results = out_tensor.into_iter().copied().collect();
            }
            _ => eprintln!(
                "Tensor {} has unsupported output type {:?}.",
                tensor_info.name, tensor_info.element_kind,
            ),
        }
    }
    let time_taken = inf_start.elapsed();
    let max = results
        .into_iter()
        .enumerate()
        .fold((0, -1.0), |acc, x| match x.1 > acc.1 {
            true => x,
            false => acc,
        });
    println!(
        "[Image analysis] max value index: {} value: {}",
        max.0, max.1
    );
    println!("Took {}ms", time_taken.as_millis());
}
