#[allow(dead_code, unused_imports)]
#[path = "../../target/flatbuffers/NetworkTestRequest_generated.rs"]
mod network_test_request;

#[allow(dead_code, unused_imports)]
#[path = "../../target/flatbuffers/NetworkTestResponse_generated.rs"]
mod network_test_response;

use crate::flow::file_identifier::{FileIdentifier, IdentifierType, ParsedFileIdentifier};
use crate::flow::Frame;
use crate::flow::uid::UID;
use crate::flow::Result;
use flatbuffers::{FlatBufferBuilder, FLATBUFFERS_MAX_BUFFER_SIZE};

use super::FlowMessage;

const NETWORK_TEST_REQUEST_IDENTIFIER: ParsedFileIdentifier = ParsedFileIdentifier {
    file_identifier: 0x3f4551,
    inner_wrapper: IdentifierType::None,
    outer_wrapper: IdentifierType::None,
    file_identifier_name: Some("NetworkTestRequest"),
};

fn serialize_request(endpoint_token: UID, response_token: UID, request_len: u32, reply_size: u32) -> Result<Frame> {
    use network_test_request::{ReplyPromise, ReplyPromiseArgs, NetworkTestRequest, NetworkTestRequestArgs, FakeRoot, FakeRootArgs};
    let mut builder = FlatBufferBuilder::with_capacity(usize::min(
        128 + (request_len as usize),
        FLATBUFFERS_MAX_BUFFER_SIZE,
    ));
    let request_len: usize = request_len.try_into()?;
    builder.start_vector::<u8>(request_len);
    for _i in 0..request_len {
        builder.push('.' as u8);
    }
    let payload = Some(builder.end_vector(request_len));
    let uid = network_test_request::UID::new(response_token.uid[0], response_token.uid[1]);
    let uid = Some(&uid);
    let reply_promise = Some(ReplyPromise::create(&mut builder, &ReplyPromiseArgs { uid }));
    let network_test_request = Some(NetworkTestRequest::create(&mut builder, &NetworkTestRequestArgs{ payload, reply_size, reply_promise }));
    let fake_root = FakeRoot::create(&mut builder, &FakeRootArgs { network_test_request });
    builder.finish(fake_root, Some("myfi"));
    let (mut payload, offset) = builder.collapse();
    FileIdentifier::new(4146513)?
        .rewrite_flatbuf(&mut payload[offset..])?;
    // println!("reply: {:x?}", builder.finished_data());
    Ok(Frame::new(endpoint_token, payload, offset))

}

fn serialize_response(token: UID, reply_size: usize) -> Result<Frame> {
    use network_test_response::{NetworkTestResponse, NetworkTestResponseArgs, EnsureTable, EnsureTableArgs, ErrorOr, FakeRoot, FakeRootArgs};
    let mut builder = FlatBufferBuilder::with_capacity(usize::min(
        128 + (reply_size),
        FLATBUFFERS_MAX_BUFFER_SIZE,
    ));
    builder.start_vector::<u8>(reply_size);
    for _i in 0..reply_size {
        builder.push('.' as u8);
    }
    let payload = builder.end_vector(reply_size);

    let network_test_response = NetworkTestResponse::create(
        &mut builder,
        &NetworkTestResponseArgs {
            payload: Some(payload),
        },
    );
    let ensure_table = EnsureTable::create(
        &mut builder,
        &EnsureTableArgs {
            network_test_response: Some(network_test_response),
        },
    );
    let fake_root = FakeRoot::create(
        &mut builder,
        &FakeRootArgs {
            error_or_type: ErrorOr::EnsureTable,
            error_or: Some(ensure_table.as_union_value()),
        },
    );
    builder.finish(fake_root, Some("myfi"));
    let (mut payload, offset) = builder.collapse();
    FileIdentifier::new(14465374)?
        .to_error_or()?
        .rewrite_flatbuf(&mut payload[offset..])?;
    // println!("reply: {:x?}", builder.finished_data());
    Ok(Frame::new(token, payload, offset))
}

pub async fn handle(request: FlowMessage) -> Result<Option<FlowMessage>> {
    request
        .file_identifier()
        .ensure_expected(NETWORK_TEST_REQUEST_IDENTIFIER)?;
    // println!("frame: {:?}", frame.payload);
    let fake_root = network_test_request::root_as_fake_root(request.frame.payload())?;
    let network_test_request = fake_root.network_test_request().unwrap();
    // println!("Got: {:?}", network_test_request);
    let reply_promise = network_test_request.reply_promise().unwrap();

    //   tokio::time::sleep(core::time::Duration::from_millis(1)).await;

    let uid = reply_promise.uid().unwrap();
    let uid = UID {
        uid: [uid.first(), uid.second()],
    };

    let frame = serialize_response(
        uid,
        network_test_request.reply_size().try_into()?,
    )?;
    Ok(Some(FlowMessage { frame }))
}