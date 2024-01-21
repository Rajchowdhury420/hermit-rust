fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("protobufs/pb_agenttasks.proto")?;
    tonic_build::compile_protos("protobufs/pb_common.proto")?;
    tonic_build::compile_protos("protobufs/pb_operations.proto")?;

    tonic_build::compile_protos("protobufs/pb_hermitrpc.proto")?;
    Ok(())
}