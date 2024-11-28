fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../qpmu-plugin/proto/plugin.proto")?;
    Ok(())
}
