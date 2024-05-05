fn main() {
    tonic_build::configure()
        .build_server(false)
        .include_file("yandex.rs")
        .compile(
            &[
                "yandex/cloud/iam/v1/iam_token_service.proto",
                "yandex/cloud/ai/stt/v2/stt_service.proto",
                "yandex/cloud/operation/operation_service.proto",
                "yandex/cloud/ai/foundation_models/v1/text_generation/text_generation_service.proto",
            ],
            &[
                "cloudapi/",
                "cloudapi/third_party/googleapis/",
                "yandex/cloud/api/",
            ],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
