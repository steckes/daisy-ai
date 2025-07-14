use burn_import::onnx::{ModelGen, RecordType};

fn main() {
    // Generate the model code from the ONNX file.
    ModelGen::new()
        .input("src/model/sine.onnx")
        .out_dir("model/")
        .record_type(RecordType::Bincode)
        .embed_states(true)
        .run_from_script();
}
