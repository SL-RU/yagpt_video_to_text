fn main() {
    let proto_files = &[
        "./cloudapi/yandex/cloud/iam/v1/iam_token_service.proto",
        "./cloudapi/yandex/cloud/ai/stt/v2/stt_service.proto",
        "./cloudapi/yandex/cloud/operation/operation_service.proto",
        "./cloudapi/yandex/cloud/ai/foundation_models/v1/text_generation/text_generation_service.proto",
    ];

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .include_file("yandex.rs")
        .compile(
            proto_files,
            &[
                "./cloudapi/",
                "./cloudapi/third_party/googleapis/",
            ],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
