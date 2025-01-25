fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../covey-plugin/proto/plugin.proto")?;
    Ok(())
}
